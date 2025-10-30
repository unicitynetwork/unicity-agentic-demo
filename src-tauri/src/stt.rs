use tauri::{AppHandle, Emitter};
use tracing::{info, debug, error};
use std::sync::{Mutex, Arc};
use once_cell::sync::Lazy;
use cpal::traits::{HostTrait, DeviceTrait};
use crate::audio_capture::AudioCapture;
use crate::audio_processor::{AudioProcessor, VoiceActivityDetector};
use crate::whisper_model::WhisperModel;
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

pub struct SpeechRecognizer {
    app_handle: AppHandle,
    recognizer: Arc<Mutex<Option<SpeechRecognizerInner>>>,
}

struct SpeechRecognizerInner {
    is_active: bool,
    audio_capture: Option<AudioCapture>,
    audio_processor: Option<AudioProcessor>,
    whisper_model: Option<WhisperModel>,
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
        match self.initialize_recognition().await {
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
            if let Some(inner) = recognizer.as_ref() {
                if !inner.is_active {
                    info!("🛑 Speech recognition not active");
                    return Ok(());
                }
            }
        }

        // Stop recognition
        *self.recognizer.lock().unwrap() = None;
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
        let device = candle_core::Device::Cpu; // TODO: Add GPU support
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
        info!("🔄 Starting audio processing loop");

        std::thread::spawn(move || {
            let mut last_transcription_time = std::time::Instant::now();
            let mut last_cleanup_time = std::time::Instant::now();

            // Use circular buffer with fixed size
            let mut audio_buffer = CircularBuffer::new(MAX_AUDIO_BUFFER_SIZE);

            loop {
                // Pull a snapshot of state and grab any available audio without holding the lock longer than needed
                let (is_active, audio_data) = {
                    let mut guard = recognizer.lock().unwrap();
                    let inner_opt = guard.as_mut();
                    // If recognizer was torn down, exit the loop
                    if inner_opt.is_none() {
                        (false, Vec::<f32>::new())
                    } else {
                        let inner = inner_opt.unwrap();
                        if !inner.is_active {
                            (false, Vec::<f32>::new())
                        } else {
                            // Try receiving a chunk from the capture (non-blocking)
                            if let Some(capture) = &mut inner.audio_capture {
                                match capture.try_recv() {
                                    Ok(data) => (true, data),
                                    Err(e) => {
                                        error!("audio try_recv error: {}", e);
                                        (true, Vec::new())
                                    }
                                }
                            } else {
                                (true, Vec::new())
                            }
                        }
                    }
                };

                if !is_active {
                    // Clean up buffer when stopping recognition
                    audio_buffer.clear();
                    break;
                }

                if audio_data.is_empty() {
                    std::thread::sleep(std::time::Duration::from_millis(AUDIO_PROCESSING_INTERVAL_MS));
                    continue;
                }

                // Add to circular buffer
                audio_buffer.extend(&audio_data);

                // Voice activity detection + streaming transcription
                let did_speech_and_processed = {
                    let mut guard = recognizer.lock().unwrap();
                    let inner = match guard.as_mut() {
                        Some(i) => i,
                        None => break,
                    };

                    if inner.vad.is_speech(&audio_data) {
                        if let Some(processor) = &mut inner.audio_processor {
                            match processor.process_audio(&audio_data) {
                                Ok(chunks) => {
                                    for chunk in chunks {
                                        if let Some(model) = &mut inner.whisper_model {
                                            match model.transcribe_streaming(&chunk) {
                                                Ok(segments) => {
                                                    for segment in segments {
                                                        if !segment.is_empty() && segment != inner.last_partial {
                                                            info!("Partial transcription: {}", segment);
                                                            let _ = app_handle.emit("stt://partial", &segment);
                                                            inner.last_partial = segment.clone();
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Transcription error: {}", e);
                                                    let _ = app_handle.emit("stt://error", format!("transcription error: {}", e));
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Audio processing error: {}", e);
                                    let _ = app_handle.emit("stt://error", format!("audio processing error: {}", e));
                                }
                            }
                        }
                        true
                    } else {
                        false
                    }
                };

                // Periodic buffer maintenance (circular buffer overwrites old data, this is informational)
                if last_cleanup_time.elapsed() >= std::time::Duration::from_secs(BUFFER_CLEANUP_INTERVAL_SECS) {
                    if audio_buffer.len() > MAX_AUDIO_BUFFER_SIZE {
                        debug!("Audio buffer at capacity, old data is being overwritten");
                    }
                    last_cleanup_time = std::time::Instant::now();
                }

                // Periodic final transcription (flush the processor)
                if last_transcription_time.elapsed() >= std::time::Duration::from_secs(FINAL_TRANSCRIPTION_INTERVAL_SECS) || did_speech_and_processed {
                    let maybe_final_text = {
                        let mut guard = recognizer.lock().unwrap();
                        let inner = match guard.as_mut() { Some(i) => i, None => break };
                        if let Some(processor) = &mut inner.audio_processor {
                            if let Ok(Some(resampled)) = processor.flush() {
                                if let Some(model) = &mut inner.whisper_model {
                                    match model.transcribe(&resampled) {
                                        Ok(text) => Some(text),
                                        Err(e) => {
                                            error!("Final transcription error: {}", e);
                                            let _ = app_handle.emit("stt://error", format!("transcription error: {}", e));
                                            None
                                        }
                                    }
                                } else { None }
                            } else { None }
                        } else { None }
                    };

                    if let Some(text) = maybe_final_text {
                        if !text.is_empty() {
                            let mut guard = recognizer.lock().unwrap();
                            if let Some(inner) = guard.as_mut() {
                                if text != inner.last_partial {
                                    let _ = app_handle.emit("stt://final", &text);
                                    inner.last_partial = text.clone();
                                }
                            }
                        }
                        last_transcription_time = std::time::Instant::now();
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(MAIN_LOOP_INTERVAL_MS));
            }
        });
    }

    fn start_processing_loop(&self) {
        Self::process_audio_loop(self.app_handle.clone(), self.recognizer.clone());
    }
}
