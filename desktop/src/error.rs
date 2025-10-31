use thiserror::Error;

/// Custom error type for Whisper implementation
#[derive(Error, Debug)]
pub enum WhisperError {
    #[error("Model error: {0}")]
    Model(String),

    #[error("Audio processing error: {0}")]
    AudioProcessing(String),

    #[error("Audio capture error: {0}")]
    AudioCapture(String),

    #[error("Speech recognition error: {0}")]
    SpeechRecognition(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Candle error: {0}")]
    Candle(#[from] candle_core::Error),

    #[error("HF Hub error: {0}")]
    HfHub(#[from] hf_hub::api::sync::ApiError),

    #[error("Tokenizer error: {0}")]
    Tokenizer(String),

    #[error("CPAL error: {0}")]
    Cpal(String),

    #[error("Resampling error: {0}")]
    Resampling(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("SurrealDB error: {0}")]
    SurrealDb(#[from] surrealdb::Error),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl WhisperError {
    pub fn model<S: Into<String>>(msg: S) -> Self {
        Self::Model(msg.into())
    }

    pub fn audio_processing<S: Into<String>>(msg: S) -> Self {
        Self::AudioProcessing(msg.into())
    }

    pub fn audio_capture<S: Into<String>>(msg: S) -> Self {
        Self::AudioCapture(msg.into())
    }

    pub fn speech_recognition<S: Into<String>>(msg: S) -> Self {
        Self::SpeechRecognition(msg.into())
    }

    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }

    pub fn tokenizer<S: Into<String>>(msg: S) -> Self {
        Self::Tokenizer(msg.into())
    }

    pub fn cpal<S: Into<String>>(msg: S) -> Self {
        Self::Cpal(msg.into())
    }

    pub fn resampling<S: Into<String>>(msg: S) -> Self {
        Self::Resampling(msg.into())
    }

    pub fn environment<S: Into<String>>(msg: S) -> Self {
        Self::Environment(msg.into())
    }

    pub fn database<S: Into<String>>(msg: S) -> Self {
        Self::Database(msg.into())
    }

    pub fn channel<S: Into<String>>(msg: S) -> Self {
        Self::Channel(msg.into())
    }

    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
}

// Conversion from anyhow::Error
impl From<anyhow::Error> for WhisperError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

// Conversion from tokenizers::Error
impl From<tokenizers::Error> for WhisperError {
    fn from(err: tokenizers::Error) -> Self {
        Self::Tokenizer(err.to_string())
    }
}

// Conversion from cpal::DevicesError
impl From<cpal::DevicesError> for WhisperError {
    fn from(err: cpal::DevicesError) -> Self {
        Self::Cpal(err.to_string())
    }
}

// Conversion from cpal::SupportedStreamConfigsError
impl From<cpal::SupportedStreamConfigsError> for WhisperError {
    fn from(err: cpal::SupportedStreamConfigsError) -> Self {
        Self::Cpal(err.to_string())
    }
}

// Conversion from cpal::DefaultStreamConfigError
impl From<cpal::DefaultStreamConfigError> for WhisperError {
    fn from(err: cpal::DefaultStreamConfigError) -> Self {
        Self::Cpal(err.to_string())
    }
}

// Conversion from rubato::ResamplerConstructionError
impl From<rubato::ResamplerConstructionError> for WhisperError {
    fn from(err: rubato::ResamplerConstructionError) -> Self {
        Self::Resampling(err.to_string())
    }
}

// Conversion from cpal::BuildStreamError
impl From<cpal::BuildStreamError> for WhisperError {
    fn from(err: cpal::BuildStreamError) -> Self {
        Self::Cpal(err.to_string())
    }
}

// Conversion from cpal::PlayStreamError
impl From<cpal::PlayStreamError> for WhisperError {
    fn from(err: cpal::PlayStreamError) -> Self {
        Self::Cpal(err.to_string())
    }
}

// Conversion from cpal::StreamError
impl From<cpal::StreamError> for WhisperError {
    fn from(err: cpal::StreamError) -> Self {
        Self::Cpal(err.to_string())
    }
}

// Conversion from rubato::ResampleError
impl From<rubato::ResampleError> for WhisperError {
    fn from(err: rubato::ResampleError) -> Self {
        Self::Resampling(err.to_string())
    }
}

// Conversion from std::sync::mpsc::TryRecvError
impl From<std::sync::mpsc::TryRecvError> for WhisperError {
    fn from(err: std::sync::mpsc::TryRecvError) -> Self {
        Self::Channel(err.to_string())
    }
}

// Conversion from std::sync::mpsc::SendError<Vec<f32>>
impl From<std::sync::mpsc::SendError<Vec<f32>>> for WhisperError {
    fn from(err: std::sync::mpsc::SendError<Vec<f32>>) -> Self {
        Self::Channel(err.to_string())
    }
}

// Conversion from std::env::VarError
impl From<std::env::VarError> for WhisperError {
    fn from(err: std::env::VarError) -> Self {
        Self::Environment(err.to_string())
    }
}

/// Result type alias for convenience
pub type WhisperResult<T> = Result<T, WhisperError>;
