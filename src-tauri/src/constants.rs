// Constants for the Whisper implementation

// Audio processing constants
pub const WHISPER_SAMPLE_RATE: u32 = 16000; // Whisper's required sample rate (16kHz)
pub const AUDIO_BUFFER_SIZE_SECONDS: usize = 30; // Audio buffer size in seconds
pub const AUDIO_CHUNK_SIZE_SECONDS: usize = 30; // Audio chunk size in seconds for processing
pub const MAX_AUDIO_BUFFER_SIZE: usize = WHISPER_SAMPLE_RATE as usize * AUDIO_BUFFER_SIZE_SECONDS; // 30 seconds of audio at 16kHz
pub const AUDIO_CHUNK_SIZE: usize = WHISPER_SAMPLE_RATE as usize * AUDIO_CHUNK_SIZE_SECONDS; // 30 seconds at 16kHz

// Resampling constants
pub const RESAMPLE_BUFFER_SIZE: usize = 1024; // Buffer size for resampling
pub const RESAMPLE_RATIO_MULTIPLIER: f64 = 10.0; // Multiplier for resampling ratio calculation

// Voice Activity Detection (VAD) constants
pub const VAD_THRESHOLD: f32 = 0.01; // Energy threshold for voice activity detection
pub const VAD_WINDOW_SIZE: usize = 10; // Window size for VAD energy calculation

// Timing constants
pub const AUDIO_PROCESSING_INTERVAL_MS: u64 = 10; // Interval in milliseconds for audio processing loop
pub const BUFFER_CLEANUP_INTERVAL_SECS: u64 = 10; // Interval in seconds for buffer cleanup
pub const FINAL_TRANSCRIPTION_INTERVAL_SECS: u64 = 5; // Interval in seconds for final transcription
pub const MAIN_LOOP_INTERVAL_MS: u64 = 100; // Interval in milliseconds for main processing loop

// Mel filter constants
pub const NUM_FFT: usize = 400; // Number of FFT points for mel spectrogram
pub const MEL_FMIN: f64 = 0.0; // Minimum frequency for mel filter bank
pub const MEL_FMAX_MULTIPLIER: f64 = 0.5; // Multiplier for maximum frequency (sample_rate * 0.5)

// Mel scale conversion constants
pub const MEL_HZ_TO_MEL_CONST: f64 = 2595.0; // Constant for Hz to Mel conversion
pub const MEL_HZ_TO_MEL_DIVISOR: f64 = 700.0; // Divisor for Hz to Mel conversion
pub const MEL_MEL_TO_HZ_MULTIPLIER: f64 = 700.0; // Multiplier for Mel to Hz conversion
pub const MEL_MEL_TO_HZ_BASE: f64 = 10.0; // Base for Mel to Hz conversion
pub const MEL_MEL_TO_HZ_DIVISOR: f64 = 2595.0; // Divisor for Mel to Hz conversion

// Application constants
pub const INITIAL_USDT_BALANCE: u64 = 100_000; // Initial USDT balance for the application
pub const DEFAULT_TRANSACTION_HISTORY_LIMIT: u32 = 50; // Default limit for transaction history queries