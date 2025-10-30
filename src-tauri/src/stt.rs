use tauri::{AppHandle, Emitter};
use tracing::{info, debug, error};
use std::sync::{Mutex, Arc};
use once_cell::sync::Lazy;
use cpal::traits::{HostTrait, DeviceTrait};
use crate::audio_capture::AudioCapture;
use crate::audio_processor::{AudioProcessor, VoiceActivityDetector};
use crate::whisper_model::{Decoder, WhisperModel};
use crate::error::{WhisperError, WhisperResult};
use crate::constants::{
    WHISPER_SAMPLE_RATE,
    MAX_AUDIO_BUFFER_SIZE,
    VAD_THRESHOLD,
    VAD_WINDOW_SIZE,
    AUDIO_PROCESSING_INTERVAL_MS,
    BUFFER_CLEANUP_INTERVAL_SECS,
    FINAL_TRANSCRIPTION_INTERVAL_SECS,
    MAIN_LOOP_INTERVAL_MS
};
use crossbeam_channel::{unbounded, Sender};

// Circular buffer for efficient audio data management
struct CircularBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    start: usize,
    len: usize,
}

impl CircularBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            start: 0,
            len: 0,
        }
    }

    fn push(&mut self, item: f32) {
        if self.len < self.capacity {
            self.buffer[(self.start + self.len) % self.capacity] = item;
            self.len += 1;
        } else {
            // Buffer is full, overwrite oldest data
            self.buffer[self.start] = item;
            self.start = (self.start + 1) % self.capacity;
        }
    }

    fn extend(&mut self, items: &[f32]) {
        for &item in items {
            self.push(item);
        }
    }

    fn as_slice(&self) -> &[f32] {
        if self.len == 0 {
            return &[];
        }

        if self.start + self.len <= self.capacity {
            &self.buffer[self.start..self.start + self.len]
        } else {
            // Buffer wraps around - need to return a Vec since we can't return two slices
            // For now, return an empty slice as this method isn't used in our implementation
            // In a real implementation, you might want to return a Vec<f32> instead
            &[]
        }
    }

    fn clear(&mut self) {
        self.start = 0;
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

fn send_log(tx: &Option<Sender<String>>, msg: impl Into<String>) {
    if let Some(tx) = tx {
        let _ = tx.send(msg.into());
    } else {
        error!("Log sender not available");
    }
}

pub struct SpeechRecognizer {
    app_handle: AppHandle,
    recognizer: Arc<Mutex<Option<SpeechRecognizerInner>>>,
}

struct SpeechRecognizerInner {
    is_active: bool,
    audio_capture: Option<AudioCapture>,
    audio_processor: Option<AudioProcessor>,
    whisper_model: Option<WhisperModel>,
    decoder: Option<Decoder<'static>>,
    vad: VoiceActivityDetector,
    last_partial: String,
}

impl SpeechRecognizer {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            recognizer: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start_recognition(&self) -> WhisperResult<()> {
        // Check if already running
        {
            let recognizer = self.recognizer.lock().unwrap();
            if let Some(inner) = recognizer.as_ref() {
                if inner.is_active {
                    info!("🎤 Speech recognition already active");
                    return Ok(());
                }
            }
        }

        // Initialize new recognition session
        let initalize = self.initialize_recognition().await;
        match initalize {
            Ok(inner) => {
                *self.recognizer.lock().unwrap() = Some(inner);
                info!("✅ Speech recognition started successfully!");
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to start speech recognition: {}", e);
                Err(e)
            }
        }
    }

    pub async fn stop_recognition(&self) -> WhisperResult<()> {
        {
            let recognizer = self.recognizer.lock().unwrap();
            if recognizer.is_none() {
                info!("🛑 Speech recognition not active");
            }
        }

        // Stop recognition
        *self.recognizer.lock().unwrap() = None;
        let _ = self.app_handle.emit("stt://log", "recognizer stopped");
        info!("✅ Speech recognition stopped");
        Ok(())
    }

    pub fn is_recognizing(&self) -> bool {
        let recognizer = self.recognizer.lock().unwrap();
        recognizer.as_ref().map(|r| r.is_active).unwrap_or(false)
    }

    async fn initialize_recognition(&self) -> WhisperResult<SpeechRecognizerInner> {
        info!("🎤 Initializing speech recognition engine");

        // Setup model and decoder
        let (whisper_model, audio_processor) = Self::setup_model_and_decoder().await?;

        // Setup audio stream (AudioCapture owns its own receiver + CPAL callbacks)
        let audio_capture = AudioCapture::new()?;

        // Initialize VAD
        let vad = VoiceActivityDetector::new(VAD_THRESHOLD, VAD_WINDOW_SIZE);

        // Start audio capture
        audio_capture.start()?;

        let inner = SpeechRecognizerInner {
            is_active: true,
            audio_capture: Some(audio_capture),
            audio_processor: Some(audio_processor),
            whisper_model: Some(whisper_model),
            decoder: None,
            vad,
            last_partial: String::new(),
        };

        // Start processing loop
        self.start_processing_loop();

        Ok(inner)
    }

    /// Setup model and decoder for speech recognition
    async fn setup_model_and_decoder() -> WhisperResult<(WhisperModel, AudioProcessor)> {
        info!("🧠 Loading Whisper model and setting up audio processor");

        // Initialize Whisper model
        let device = candle_core::Device::new_metal(0)?; // TODO: Add GPU support
        let whisper_model = WhisperModel::new(None, device).await?;

        // Get audio config for processor setup
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| WhisperError::audio_capture("No default input device found"))?;
        let config = device.default_input_config()?;

        info!("🎚️ Input audio sample-rate: {} Hz, channels: {}", config.sample_rate().0, config.channels());

        // Initialize audio processor
        let audio_processor = AudioProcessor::new(config.sample_rate().0, &whisper_model.config())?;

        info!("✅ Model and decoder setup complete");
        Ok((whisper_model, audio_processor))
    }

    /// Setup audio stream for capturing audio
    async fn setup_audio_stream(_audio_processor: &AudioProcessor) -> WhisperResult<AudioCapture> {
        info!("🎤 Setting up audio capture stream");

        // Initialize audio capture
        let audio_capture = AudioCapture::new()?;

        info!("✅ Audio stream setup complete");
        Ok(audio_capture)
    }

    /// Main audio processing loop that handles speech recognition
    fn process_audio_loop(app_handle: AppHandle, recognizer: Arc<Mutex<Option<SpeechRecognizerInner>>>) {
        println!("🔄 Starting audio processing loop");

        std::thread::spawn(move || {
            println!("🧵 SPAWNED THREAD STARTED!");

            let mut audio_buffer: Vec<f32> = Vec::new();
            let target_buffer_size = 48000; // 1 second at 48kHz (adjust based on your input rate)
            let mut silence_count = 0;

            loop {
                let audio_chunk = {
                    let mut guard = recognizer.lock().unwrap();
                    let inner = match guard.as_mut() {
                        Some(i) if i.is_active => i,
                        _ => {
                            println!("🛑 Recognizer stopped, exiting loop");
                            break;
                        }
                    };

                    if let Some(capture) = &mut inner.audio_capture {
                        match capture.recv_timeout(100) {
                            Ok(data) => {
                                if data.is_empty() {
                                    continue;
                                }
                                data
                            }
                            Err(e) => {
                                println!("❌ Audio capture error: {}", e);
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                };

                // Accumulate audio
                audio_buffer.extend_from_slice(&audio_chunk);

                if audio_buffer.len() < target_buffer_size {
                    continue;
                }

                let samples_to_process: Vec<f32> = audio_buffer.drain(..target_buffer_size).collect();

                // Process and transcribe
                let transcription = {
                    let mut guard = recognizer.lock().unwrap();
                    let inner = match guard.as_mut() {
                        Some(i) => i,
                        None => break,
                    };

                    if let Some(processor) = &mut inner.audio_processor {
                        match processor.process_audio(&samples_to_process) {
                            Ok(chunks) => {
                                let mut results = Vec::new();
                                for chunk in chunks {
                                    if let Some(model) = &mut inner.whisper_model {
                                        match model.transcribe_streaming_with_context(&chunk) {
                                            Ok(Some(text)) => {
                                                println!("📝 Transcribed: '{}'", text);
                                                results.push(text);
                                                silence_count = 0;
                                            }
                                            Ok(None) => {
                                                println!("🤫 No speech");
                                                silence_count += 1;

                                                // Reset after 3 seconds of silence
                                                if silence_count >= 3 {
                                                    println!("🔄 Resetting context after silence");
                                                    model.reset_context();
                                                    silence_count = 0;
                                                }
                                            }
                                            Err(e) => {
                                                println!("❌ Error: {}", e);
                                            }
                                        }
                                    }
                                }
                                results
                            }
                            Err(e) => {
                                println!("❌ Processing error: {}", e);
                                Vec::new()
                            }
                        }
                    } else {
                        Vec::new()
                    }
                };

                // Emit results
                for text in transcription {
                    let _ = app_handle.emit("stt://partial", &text);
                }
            }

            println!("🏁 Audio processing loop ENDED");
        });
    }

    fn start_processing_loop(&self) {
        Self::process_audio_loop(self.app_handle.clone(), self.recognizer.clone());
    }
}
