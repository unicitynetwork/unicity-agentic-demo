use kalosm::sound::*;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tracing::{error, info};
use std::path::PathBuf;
use std::env;

pub struct SpeechRecognizer {
    app_handle: AppHandle,
    is_active: Arc<Mutex<bool>>,
    model: Arc<Mutex<Option<Whisper>>>,
}

impl SpeechRecognizer {
    pub fn new(app_handle: AppHandle) -> Self {
        // Setup custom cache directory BEFORE loading any models
        // This must happen before any Kalosm/HuggingFace operations
        match setup_whisper_cache() {
            Ok(cache_dir) => {
                info!("📁 Cache directory: {}", cache_dir.display());
                let _ = app_handle.emit(
                    "stt://log",
                    format!("📁 Cache: {}", cache_dir.display())
                );
            }
            Err(e) => {
                error!("⚠️ Failed to setup cache directory: {}", e);
                let _ = app_handle.emit(
                    "stt://log",
                    format!("⚠️ Cache setup failed: {}", e)
                );
                // Continue anyway - will use default cache
            }
        }

        Self {
            app_handle,
            is_active: Arc::new(Mutex::new(false)),
            model: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start_recognition(&self) {
        // Check if already running
        {
            let is_active = self.is_active.lock().unwrap();
            if *is_active {
                info!("🎤 Speech recognition already active");
                return;
            }
        }

        // Initialize new recognition session
        self.initialize_recognition().await;
    }

    pub async fn stop_recognition(&self) {
        {
            let is_active = self.is_active.lock().unwrap();
            if !*is_active {
                info!("🛑 Speech recognition not active");
                return;
            }
        }

        // Stop recognition
        *self.is_active.lock().unwrap() = false;
        let _ = self.app_handle.emit("stt://log", "recognizer stopped");
        info!("✅ Speech recognition stopped");
    }

    pub fn is_recognizing(&self) -> bool {
        let is_active = self.is_active.lock().unwrap();
        *is_active
    }

    async fn initialize_recognition(&self) {
        info!("🎤 Initializing speech recognition engine");
        let _ = self.app_handle.emit("stt://log", "🎤 Initializing speech recognition engine");

        let needs_loading = {
            let guard = self.model.lock().unwrap();
            guard.is_none()
        };

        // Load model if not already loaded
        if needs_loading {
            let model = WhisperBuilder::default()
                .with_source(WhisperSource::SmallEn)
                .build_with_loading_handler(|progress| match progress {
                    ModelLoadingProgress::Downloading { source, progress } => {
                        let progress = (progress.progress) as u32;
                        println!("Downloading {source} {progress}");
                    }
                    ModelLoadingProgress::Loading { progress } => {
                        let progress_percent = (progress * 100.0) as u32;
                        println!("Loading model {progress_percent}%");
                    }
                })
                .await;

            match model {
                Ok(m) => {
                    *self.model.lock().unwrap() = Some(m);
                    let _ = self.app_handle.emit("stt://log", "✅ Model loaded");
                }
                Err(e) => {
                    let _ = self.app_handle.emit("stt://log", format!("❌ Model error: {}", e));
                    return;
                }
            }
        }

        *self.is_active.lock().unwrap() = true;

        // Start processing loop
        self.start_processing_loop();

        info!("✅ Model and audio setup complete");
        let _ = self.app_handle.emit("stt://log", "✅ Model and audio setup complete");
    }

    fn start_processing_loop(&self) {
        let app_handle = self.app_handle.clone();
        let is_active = self.is_active.clone();
        let model = self.model.clone();

        tokio::spawn(async move {
            // Get stream once
            let mic = MicInput::default();
            let stream = mic.stream();
            // Get the model (it's already loaded)
            let model = {
                let guard = model.lock().unwrap();
                guard.as_ref().cloned() // Need to clone the model
            }.expect("Model should be loaded");

            // Transcribe once
            let mut transcriptions = stream.transcribe(model);

            // Loop until stopped
            while let Some(segment) = transcriptions.next().await {
                // Check flag
                let is_active = *is_active.lock().unwrap();

                if !is_active { break; }

                // Process segment
                let text = segment.text().trim();
                if !text.is_empty() && segment.probability_of_no_speech() < 0.90 {
                    println!("Transcribed: {}", text);
                    let _ = app_handle.emit("stt://partial", text);
                }
            }
        });
    }
}

/// Sets up the custom cache directory for Whisper models downloaded by Kalosm.
///
/// Kalosm uses HuggingFace Hub under the hood, which respects the HF_HOME environment variable.
/// This function sets up the cache directory before any models are loaded.
///
/// # Cache Directory Priority
/// 1. WHISPER_CACHE_DIR environment variable (if set)
/// 2. ~/.unicity-agentic-demo/whisper-cache (default)
/// 3. Current directory/.unicity-agentic-demo/whisper-cache (fallback)
///
/// # Returns
/// The path to the cache directory that was configured
pub fn setup_whisper_cache() -> Result<PathBuf, std::io::Error> {
    let cache_dir: PathBuf = if let Some(dir) = env::var_os("WHISPER_CACHE_DIR").map(PathBuf::from) {
        dir
    } else if let Some(base) = dirs::home_dir() {
        base.join(".unicity-agentic-demo").join("whisper-cache")
    } else {
        // Final fallback – prefer not to use CWD to avoid dev reload loops
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".unicity-agentic-demo")
            .join("whisper-cache")
    };

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&cache_dir)?;

    // Set HF_HOME environment variable to our custom cache directory
    // This tells HuggingFace Hub (used by Kalosm) where to cache models
    env::set_var("HF_HOME", &cache_dir);

    info!("📁 Whisper cache directory set to: {}", cache_dir.display());

    Ok(cache_dir)
}

/// Returns the current Whisper cache directory without setting it up.
/// Returns None if the cache hasn't been configured yet.
pub fn get_whisper_cache_dir() -> Option<PathBuf> {
    env::var_os("HF_HOME").map(PathBuf::from)
}

/// Clears the Whisper model cache (deletes all cached models).
/// Use with caution - models will need to be re-downloaded.
pub fn clear_whisper_cache() -> Result<(), std::io::Error> {
    if let Some(cache_dir) = get_whisper_cache_dir() {
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir)?;
            info!("🗑️ Cleared Whisper cache at: {}", cache_dir.display());
        }
    }
    Ok(())
}

/// Gets the size of the Whisper cache directory in bytes.
pub fn get_cache_size() -> Result<u64, std::io::Error> {
    fn dir_size(path: &PathBuf) -> Result<u64, std::io::Error> {
        let mut size = 0u64;
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    size += dir_size(&path)?;
                } else {
                    size += entry.metadata()?.len();
                }
            }
        }
        Ok(size)
    }

    if let Some(cache_dir) = get_whisper_cache_dir() {
        if cache_dir.exists() {
            return dir_size(&cache_dir);
        }
    }
    Ok(0)
}

/// Formats bytes into a human-readable string (e.g., "1.5 GB")
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_cache_creates_directory() {
        let cache_dir = setup_whisper_cache().unwrap();
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1536 * 1024 * 1024), "1.50 GB");
    }
}

