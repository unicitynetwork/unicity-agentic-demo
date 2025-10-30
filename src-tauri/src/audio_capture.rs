use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, SampleFormat, SupportedStreamConfig, FromSample};
use std::sync::mpsc::{Receiver, Sender};
use tracing::{info, error, debug};
use crate::error::{WhisperError, WhisperResult};

pub struct AudioCapture {
    _stream: Stream,
    receiver: Receiver<Vec<f32>>,
}

unsafe impl Send for AudioCapture {}
unsafe impl Sync for AudioCapture {}

impl AudioCapture {
    pub fn new() -> WhisperResult<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| WhisperError::audio_capture("No default input device found"))?;

        info!("Using audio device: {}", device.name().unwrap_or("Unknown".to_string()));

        let config = device.default_input_config()?;

        debug!("Audio config: {:?}", config);

        let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();
        
        let stream = match config.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), tx)?,
            SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), tx)?,
            SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), tx)?,
            SampleFormat::I8 => build_stream::<i8>(&device, &config.into(), tx)?,
            SampleFormat::U8 => build_stream::<u8>(&device, &config.into(), tx)?,
            _ => return Err(WhisperError::audio_capture(format!("Unsupported sample format: {:?}", config.sample_format()))),
        };

        Ok(Self {
            _stream: stream,
            receiver: rx,
        })
    }

    pub fn start(&self) -> WhisperResult<()> {
        self._stream.play()?;
        info!("Audio capture started");
        Ok(())
    }

    pub fn recv(&self) -> WhisperResult<Vec<f32>> {
        match self.receiver.recv() {
            Ok(data) => Ok(data),
            Err(_) => Err(WhisperError::channel("Audio channel disconnected")),
        }
    }

    pub fn recv_timeout(&self, timeout_ms: u64) -> WhisperResult<Vec<f32>> {
        match self.receiver.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
            Ok(data) => Ok(data),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(Vec::new()),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err(WhisperError::channel("Audio channel disconnected"))
            }
        }
    }

    pub fn try_recv(&self) -> WhisperResult<Vec<f32>> {
        match self.receiver.try_recv() {
            Ok(data) => Ok(data),
            Err(std::sync::mpsc::TryRecvError::Empty) => Ok(Vec::new()),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Err(WhisperError::channel("Audio channel disconnected"))
            }
        }
    }

    pub fn set_receiver(&mut self, receiver: Receiver<Vec<f32>>) {
        self.receiver = receiver;
    }
}

fn build_stream<T>(
    device: &Device,
    config: &StreamConfig,
    sender: Sender<Vec<f32>>,
) -> WhisperResult<Stream>
where
    T: cpal::Sample + cpal::SizedSample,
    f64: FromSample<T>,
{
    let channel_count = config.channels as usize;
    let sample_rate = config.sample_rate.0 as f64;
    
    info!("Setting up audio stream: {} channels, {} Hz", channel_count, sample_rate);

    let err_fn = move |err| {
        error!("Audio stream error: {}", err);
    };

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            // Convert to mono f32
            let mono_samples: Vec<f32> = data
                .chunks(channel_count)
                .map(|chunk| {
                    // Average all channels to get mono
                    let sum: f64 = chunk.iter().map(|&sample| {
                        let sample_f64: f64 = cpal::Sample::from_sample(sample);
                        sample_f64 as f64
                    }).sum();
                    (sum / channel_count as f64) as f32
                })
                .collect();

            if !mono_samples.is_empty() {
                let _ = sender.send(mono_samples);
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

pub fn list_audio_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devices| {
            devices
                .filter_map(|d| d.name().ok())
                .collect()
        })
        .unwrap_or_default()
}