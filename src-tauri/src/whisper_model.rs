use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as m, audio, Config};
use hf_hub::{api::sync::{Api, ApiBuilder}, api::Progress};
use std::path::PathBuf;
use candle_transformers::models::whisper::audio::pcm_to_mel;
use rand::{Rng, SeedableRng};
use tokenizers::Tokenizer;
use tracing::{info, debug};
use crate::audio_processor::generate_mel_filters;
use crate::error::{WhisperError, WhisperResult};

#[derive(Clone)]
struct HubProgress {
    label: &'static str,
    current: usize,
    total: usize,
}

impl HubProgress {
    fn new(label: &'static str) -> Self {
        Self { label, current: 0, total: 0 }
    }
}

impl Progress for HubProgress {
    fn init(&mut self, size: usize, _filename: &str) {
        self.total = size;
        self.current = 0;
        info!("⬇️ downloading {} ({} bytes)", self.label, size);
    }

    fn update(&mut self, size: usize) {
        self.current += size;
        if self.total > 0 {
            let pct = (self.current as f64 / self.total as f64) * 100.0;
            debug!("{}: {}/{} ({:.1}%)", self.label, self.current, self.total, pct);
        } else {
            debug!("{}: {} bytes", self.label, self.current);
        }
    }

    fn finish(&mut self) {
        info!("✅ finished {} ({} bytes)", self.label, self.current);
    }
}

/// Trait that defines common methods for both normal and quantized Whisper models
pub trait WhisperModelTrait {
    fn config(&self) -> &Config;
    fn encoder_forward(&mut self, x: &Tensor, flush: bool) -> candle_core::Result<Tensor>;
    fn decoder_forward(&mut self, x: &Tensor, xa: &Tensor, flush: bool) -> candle_core::Result<Tensor>;
    fn decoder_final_linear(&self, x: &Tensor) -> candle_core::Result<Tensor>;
    fn reset_kv_cache(&mut self);
}

/// Implement the trait for the normal Whisper model
impl WhisperModelTrait for m::model::Whisper {
    fn config(&self) -> &Config {
        &self.config
    }

    fn encoder_forward(&mut self, x: &Tensor, flush: bool) -> candle_core::Result<Tensor> {
        self.encoder.forward(x, flush)
    }

    fn decoder_forward(&mut self, x: &Tensor, xa: &Tensor, flush: bool) -> candle_core::Result<Tensor> {
        self.decoder.forward(x, xa, flush)
    }

    fn decoder_final_linear(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        self.decoder.final_linear(x)
    }

    fn reset_kv_cache(&mut self) {
        m::model::Whisper::reset_kv_cache(self)
    }
}

/// Implement the trait for the quantized Whisper model
impl WhisperModelTrait for m::quantized_model::Whisper {
    fn config(&self) -> &Config {
        &self.config
    }

    fn encoder_forward(&mut self, x: &Tensor, flush: bool) -> candle_core::Result<Tensor> {
        self.encoder.forward(x, flush)
    }

    fn decoder_forward(&mut self, x: &Tensor, xa: &Tensor, flush: bool) -> candle_core::Result<Tensor> {
        self.decoder.forward(x, xa, flush)
    }

    fn decoder_final_linear(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        self.decoder.final_linear(x)
    }

    fn reset_kv_cache(&mut self) {
        m::quantized_model::Whisper::reset_kv_cache(self)
    }
}

pub enum Model {
    Normal(m::model::Whisper),
    Quantized(m::quantized_model::Whisper),
}

impl Model {
    pub fn config(&self) -> &Config {
        match self {
            Self::Normal(m) => m.config(),
            Self::Quantized(m) => m.config(),
        }
    }

    pub fn encoder_forward(&mut self, x: &Tensor, flush: bool) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.encoder_forward(x, flush),
            Self::Quantized(m) => m.encoder_forward(x, flush),
        }
    }

    pub fn decoder_forward(
        &mut self,
        x: &Tensor,
        xa: &Tensor,
        flush: bool,
    ) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.decoder_forward(x, xa, flush),
            Self::Quantized(m) => m.decoder_forward(x, xa, flush),
        }
    }

    pub fn decoder_final_linear(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.decoder_final_linear(x),
            Self::Quantized(m) => m.decoder_final_linear(x),
        }
    }

    pub fn reset_kv_cache(&mut self) {
        match self {
            Model::Normal(m) => m.reset_kv_cache(),
            Model::Quantized(m) => m.reset_kv_cache(),
        }
    }
}

pub struct WhisperModel {
    model: Model,
    tokenizer: Tokenizer,
    device: Device,
    config: Config,
    mel_filters: Vec<f32>,
    language_token: Option<u32>,
    suppress_tokens: Tensor,
    sot_token: u32,
    transcribe_token: u32,
    eot_token: u32,
    no_speech_token: u32,
    no_timestamps_token: u32,
}

impl WhisperModel {
    pub async fn new(model_id: Option<String>, device: Device) -> WhisperResult<Self> {
        info!("Loading Whisper model...");

        // Default to LargeV3Turbo if not specified
        let (model_id, revision) = if let Some(id) = model_id {
            (id, "main".to_string())
        } else {
            ("openai/whisper-large-v3-turbo".to_string(), "main".to_string())
        };

        // Choose a cache dir that won't trigger frontend/dev hot-reload.
        // Priority: WHISPER_CACHE_DIR env > OS home dir > fallback to temp within home
        let cache_dir: PathBuf = if let Some(dir) = std::env::var_os("WHISPER_CACHE_DIR").map(PathBuf::from) {
            dir
        } else if let Some(base) = dirs::home_dir() {
            base.join(".unicity-agentic-demo").join("whisper-cache")
        } else {
            // final fallback – prefer not to use CWD to avoid dev reload loops
            dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".unicity-agentic-demo").join("whisper-cache")
        };
        std::fs::create_dir_all(&cache_dir)?;
        info!("Whisper cache dir resolved to {} (avoids project root to prevent dev refresh)", cache_dir.display());
        let api = ApiBuilder::new().with_cache_dir(cache_dir).build()?;

        // Pin to an explicit revision so the cache key is stable
        let repo = api.repo(hf_hub::Repo::with_revision(model_id.clone(), hf_hub::RepoType::Model, revision.clone()));

        // Retrieve (from cache if present; download otherwise)
        info!("Ensuring model files exist in cache (revision: {})", revision);
        let config_path = repo.get("config.json")?;
        let tokenizer_path = repo.get("tokenizer.json")?;
        let model_path = repo.get("model.safetensors")?;

        // Load config
        let config: Config = serde_json::from_str(&std::fs::read_to_string(config_path)?)?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)?;

        // Load model
        let model = {
            let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_path], m::DTYPE, &device)? };
            Model::Normal(m::model::Whisper::load(&vb, config.clone())?)
        };

        // Generate mel filters dynamically
        let mel_filters = generate_mel_filters(config.num_mel_bins)?;

        info!("Whisper model loaded successfully");

        let no_timestamps_token = token_id(&tokenizer, m::NO_TIMESTAMPS_TOKEN)?;
        let suppress_tokens: Vec<f32> = (0..config.vocab_size as u32)
            .map(|i| {
                if config.suppress_tokens.contains(&i) {
                    f32::NEG_INFINITY
                } else {
                    0f32
                }
            })
            .collect();
        let suppress_tokens = Tensor::new(suppress_tokens.as_slice(), &device)?;
        let sot_token = token_id(&tokenizer, m::SOT_TOKEN)?;
        let transcribe_token = token_id(&tokenizer, m::TRANSCRIBE_TOKEN)?;
        let eot_token = token_id(&tokenizer, m::EOT_TOKEN)?;
        let no_speech_token = m::NO_SPEECH_TOKENS
            .iter()
            .find_map(|token| token_id(&tokenizer, token).ok())
            .ok_or_else(|| WhisperError::model("No speech token not found"))?;

        Ok(Self {
            model,
            tokenizer,
            device,
            config,
            mel_filters,
            language_token: None,
            suppress_tokens,
            sot_token,
            transcribe_token,
            eot_token,
            no_speech_token,
            no_timestamps_token,
        })
    }

    // Create a decoder that borrows self mutably
    fn create_decoder(&mut self) -> Decoder {
        Decoder {
            model: &mut self.model,
            rng: rand::rngs::StdRng::seed_from_u64(299792458),
            tokenizer: self.tokenizer.clone(),
            suppress_tokens: self.suppress_tokens.clone(),
            sot_token: self.sot_token,
            transcribe_token: self.transcribe_token,
            eot_token: self.eot_token,
            no_speech_token: self.no_speech_token,
            no_timestamps_token: self.no_timestamps_token,
            language_token: self.language_token,
        }
    }

    // Streaming transcription that maintains context
    pub fn transcribe_streaming_with_context(&mut self, audio: &[f32]) -> WhisperResult<Option<String>> {
        let mel = pcm_to_mel(&self.config, audio, &self.mel_filters);
        let mel_len = mel.len();
        let mel = Tensor::from_vec(
            mel,
            (1, self.config.num_mel_bins, mel_len / self.config.num_mel_bins),
            &self.device,
        )?;

        let mut decoder = self.create_decoder();
        let result = decoder.decode_chunk(&mel)?;

        // Update language token if it was set
        self.language_token = decoder.language_token;

        Ok(result)
    }

    // Reset KV cache (call after silence)
    pub fn reset_context(&mut self) {
        self.model.reset_kv_cache();
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

pub struct Decoder<'a> {
    model: &'a mut Model,
    pub rng: rand::rngs::StdRng,
    pub tokenizer: Tokenizer,
    pub suppress_tokens: Tensor,
    pub sot_token: u32,
    pub transcribe_token: u32,
    pub eot_token: u32,
    pub no_speech_token: u32,
    pub no_timestamps_token: u32,
    pub language_token: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum Task {
    Transcribe,
    Translate,
}

impl<'a> Decoder<'a> {
    pub fn run(&mut self, mel: &Tensor, _times: Option<(f64, f64)>) -> WhisperResult<Option<String>> {
        // Use fallback decoding with multiple temperatures
        self.decode_chunk(mel)
    }

    pub fn decode_chunk(&mut self, mel: &Tensor) -> WhisperResult<Option<String>> {
        let model = &mut self.model;

        // Encode audio
        let audio_features = model.encoder_forward(mel, false)?;

        let sample_len = model.config().max_target_positions / 2;
        let mut sum_logprob = 0f64;
        let mut no_speech_prob = f64::NAN;
        let mut tokens = vec![self.sot_token];

        // Set language token on first chunk
        if self.language_token.is_none() {
            // Auto-detect or set English
            self.language_token = Some(token_id(&self.tokenizer, "<|en|>")?);
        }

        if let Some(lang_token) = self.language_token {
            tokens.push(lang_token);
        }
        tokens.push(self.transcribe_token);
        tokens.push(self.no_timestamps_token);

        // Decode
        for i in 0..sample_len {
            let tokens_t = Tensor::new(tokens.as_slice(), mel.device())?;
            let tokens_t = tokens_t.unsqueeze(0)?;
            let ys = model.decoder_forward(&tokens_t, &audio_features, i == 0)?;

            // Check no_speech_prob on first iteration
            if i == 0 {
                let logits = model.decoder_final_linear(&ys.i(..1)?)?.i(0)?.i(0)?;
                no_speech_prob = candle_nn::ops::softmax(&logits, 0)?
                    .i(self.no_speech_token as usize)?
                    .to_scalar::<f32>()? as f64;
            }

            let (_, seq_len, _) = ys.dims3()?;
            let logits = model.decoder_final_linear(&ys.i((..1, seq_len - 1..))?)?
                .i(0)?.i(0)?;
            let logits = logits.broadcast_add(&self.suppress_tokens)?;

            // Greedy decoding
            let logits_v: Vec<f32> = logits.to_vec1()?;
            let next_token = logits_v
                .iter()
                .enumerate()
                .max_by(|(_, u), (_, v)| u.total_cmp(v))
                .map(|(i, _)| i as u32)
                .unwrap();

            tokens.push(next_token);

            let prob = candle_nn::ops::softmax(&logits, candle_core::D::Minus1)?
                .i(next_token as usize)?
                .to_scalar::<f32>()? as f64;

            if next_token == self.eot_token || tokens.len() > model.config().max_target_positions {
                break;
            }
            sum_logprob += prob.ln();
        }

        let avg_logprob = sum_logprob / tokens.len() as f64;

        // Filter out hallucinations
        const NO_SPEECH_THRESHOLD: f64 = 0.6;
        const LOGPROB_THRESHOLD: f64 = -1.0;

        if no_speech_prob > NO_SPEECH_THRESHOLD && avg_logprob < LOGPROB_THRESHOLD {
            println!("🤫 No speech detected (prob: {:.3}), skipping", no_speech_prob);
            return Ok(None);
        }

        let text = self.tokenizer
            .decode(&tokens, true)
            .map_err(|e| WhisperError::tokenizer(e.to_string()))?;

        let text = text.trim();

        // Filter out very short results (likely hallucinations)
        if text.len() < 3 {
            println!("⏭️ Skipping short result: '{}'", text);
            return Ok(None);
        }

        Ok(Some(text.to_string()))
    }

    pub fn reset(&mut self) {
        self.model.reset_kv_cache();
    }
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub start: f64,
    pub duration: f64,
    pub dr: DecodingResult,
}

#[derive(Debug, Clone)]
pub struct DecodingResult {
    pub tokens: Vec<u32>,
    pub text: String,
    pub avg_logprob: f64,
    pub no_speech_prob: f64,
    pub temperature: f64,
    pub compression_ratio: f64,
}

pub fn token_id(tokenizer: &Tokenizer, token: &str) -> WhisperResult<u32> {
    match tokenizer.token_to_id(token) {
        None => Err(WhisperError::tokenizer(format!("no token-id for {}", token))),
        Some(id) => Ok(id),
    }
}


fn softmax<T: candle_core::WithDType>(logits: &Tensor, d: candle_core::D) -> candle_core::Result<Tensor> {
    candle_nn::ops::softmax(logits, d)
}