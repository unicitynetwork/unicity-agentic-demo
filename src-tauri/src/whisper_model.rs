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

        Ok(Self {
            model,
            tokenizer,
            device,
            config,
            mel_filters,
        })
    }

    pub fn transcribe(&mut self, audio: &[f32]) -> WhisperResult<String> {
        // Convert audio to mel spectrogram
        let mel = pcm_to_mel(&self.config, audio, &self.mel_filters);
        let mel_len = mel.len();
        let mel = Tensor::from_vec(
            mel,
            (1, self.config.num_mel_bins, mel_len / self.config.num_mel_bins),
            &self.device,
        )?;

        // Create decoder
        let mut decoder = self.create_decoder(None, None, false, false)?;

        // Run transcription
        let segments = decoder.run(&mel, None)?;

        // Combine all segments
        let text = segments
            .into_iter()
            .map(|s| s.dr.text)
            .collect::<Vec<String>>()
            .join(" ");

        Ok(text.trim().to_string())
    }

    pub fn transcribe_streaming(&mut self, audio: &[f32]) -> WhisperResult<Vec<String>> {
        // Convert audio to mel spectrogram
        let mel = pcm_to_mel(&self.config, audio, &self.mel_filters);
        let mel_len = mel.len();
        let mel = Tensor::from_vec(
            mel,
            (1, self.config.num_mel_bins, mel_len / self.config.num_mel_bins),
            &self.device,
        )?;

        // Create decoder for streaming
        let mut decoder = self.create_decoder(None, None, false, true)?;

        // Run transcription
        let segments = decoder.run(&mel, None)?;

        // Return individual segments for streaming
        Ok(segments
            .into_iter()
            .map(|s| s.dr.text.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    fn create_decoder(
        &mut self,
        language_token: Option<u32>,
        task: Option<Task>,
        timestamps: bool,
        verbose: bool,
    ) -> WhisperResult<Decoder> {
        let no_timestamps_token = token_id(&self.tokenizer, m::NO_TIMESTAMPS_TOKEN)?;

        // Suppress tokens
        let suppress_tokens: Vec<f32> = (0..self.config.vocab_size as u32)
            .map(|i| {
                if self.config.suppress_tokens.contains(&i)
                    || timestamps && i == no_timestamps_token
                {
                    f32::NEG_INFINITY
                } else {
                    0f32
                }
            })
            .collect();
        let suppress_tokens = Tensor::new(suppress_tokens.as_slice(), &self.device)?;

        let sot_token = token_id(&self.tokenizer, m::SOT_TOKEN)?;
        let transcribe_token = token_id(&self.tokenizer, m::TRANSCRIBE_TOKEN)?;
        let eot_token = token_id(&self.tokenizer, m::EOT_TOKEN)?;

        Ok(Decoder {
            model: &mut self.model,
            rng: rand::rngs::StdRng::seed_from_u64(299792458),
            tokenizer: self.tokenizer.clone(),
            suppress_tokens,
            sot_token,
            transcribe_token,
            eot_token,
            language_token,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

pub struct Decoder<'a> {
    model: &'a mut Model,
    rng: rand::rngs::StdRng,
    tokenizer: Tokenizer,
    suppress_tokens: Tensor,
    sot_token: u32,
    transcribe_token: u32,
    eot_token: u32,
    language_token: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum Task {
    Transcribe,
    Translate,
}

impl<'a> Decoder<'a> {
    pub fn run(&mut self, mel: &Tensor, _times: Option<(f64, f64)>) -> WhisperResult<Vec<Segment>> {
        // Use fallback decoding with multiple temperatures
        self.decode_with_fallback(mel)
    }

    /// Decode with temperature fallback sampling
    /// Tries multiple temperatures from lowest to highest until quality criteria are met
    fn decode_with_fallback(&mut self, mel: &Tensor) -> WhisperResult<Vec<Segment>> {
        let temperatures = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        let mut best_result: Option<Segment> = None;

        for &temperature in &temperatures {
            // Reset model state for each attempt
            self.model.reset_kv_cache();

            match self.decode_with_temperature(mel, temperature) {
                Ok(segment) => {
                    let dr = &segment.dr;

                    // Calculate compression ratio
                    let compression_ratio = self.calculate_compression_ratio(&dr.text);

                    // Quality checks
                    let avg_logprob_ok = dr.avg_logprob > -1.0; // Reasonable threshold
                    let compression_ok = compression_ratio < 2.4; // Not too repetitive

                    // For temperature 0.0 (greedy), accept if avg_logprob is reasonable
                    // For higher temperatures, require both criteria
                    let quality_ok = if temperature == 0.0 {
                        avg_logprob_ok
                    } else {
                        avg_logprob_ok && compression_ok
                    };

                    if quality_ok {
                        // Create updated segment with compression ratio
                        let mut updated_segment = segment.clone();
                        updated_segment.dr.compression_ratio = compression_ratio;
                        return Ok(vec![updated_segment]);
                    }

                    // Keep track of best result based on avg_logprob
                    if best_result.is_none() || dr.avg_logprob > best_result.as_ref().unwrap().dr.avg_logprob {
                        let mut updated_segment = segment.clone();
                        updated_segment.dr.compression_ratio = compression_ratio;
                        best_result = Some(updated_segment);
                    }
                }
                Err(e) => {
                    debug!("Failed to decode with temperature {}: {}", temperature, e);
                    // Continue to next temperature
                }
            }
        }

        // If no result passed quality checks, return the best one we have
        if let Some(segment) = best_result {
            info!("Using fallback result with avg_logprob: {}", segment.dr.avg_logprob);
            return Ok(vec![segment]);
        }

        // If everything failed, return an error
        Err(WhisperError::model("All decoding attempts failed"))
    }

    /// Decode with a specific temperature
    fn decode_with_temperature(&mut self, mel: &Tensor, temperature: f32) -> WhisperResult<Segment> {
        let model = &mut self.model;
        let audio_features = model.encoder_forward(mel, true)?;

        let sample_len = model.config().max_target_positions / 2;
        let mut sum_logprob = 0f64;
        let mut tokens = vec![self.sot_token];

        if let Some(language_token) = self.language_token {
            tokens.push(language_token);
        }

        // Always use transcribe task (simplified)
        tokens.push(self.transcribe_token);

        for i in 0..sample_len {
            let tokens_t = Tensor::new(tokens.as_slice(), mel.device())?;
            let tokens_t = tokens_t.unsqueeze(0)?;
            let ys = model.decoder_forward(&tokens_t, &audio_features, i == 0)?;

            let (_, seq_len, _) = ys.dims3()?;
            let logits = model
                .decoder_final_linear(&ys.i((..1, seq_len - 1..))?)?
                .i(0)?
                .i(0)?;

            let logits = logits.broadcast_add(&self.suppress_tokens)?;

            // Temperature-based sampling
            let next_token = if temperature > 0.0 {
                let logits_v: Vec<f32> = logits.to_vec1()?;
                let scaled_logits: Vec<f32> = logits_v.iter().map(|&x| x / temperature).collect();
                let exp_logits: Vec<f32> = scaled_logits.iter().map(|&x| x.exp()).collect();
                let sum_exp: f32 = exp_logits.iter().sum();
                let probs: Vec<f32> = exp_logits.iter().map(|&x| x / sum_exp).collect();

                // Simple sampling based on probabilities
                let mut cumsum = 0.0;
                let random_val: f32 = self.rng.gen();
                let mut selected = (probs.len() - 1) as u32;
                for (i, &prob) in probs.iter().enumerate() {
                    cumsum += prob;
                    if random_val <= cumsum {
                        selected = i as u32;
                        break;
                    }
                }
                selected
            } else {
                // Greedy decoding (temperature = 0)
                let logits_v: Vec<f32> = logits.to_vec1()?;
                logits_v
                    .iter()
                    .enumerate()
                    .max_by(|(_, u), (_, v)| u.total_cmp(v))
                    .map(|(i, _)| i as u32)
                    .unwrap()
            };

            tokens.push(next_token);
            let prob = softmax::<f32>(&logits, candle_core::D::Minus1)?
                .i(next_token as usize)?
                .to_scalar::<f32>()? as f64;

            if next_token == self.eot_token || tokens.len() > model.config().max_target_positions {
                break;
            }
            sum_logprob += prob.ln();
        }

        let text = self
            .tokenizer
            .decode(&tokens, true)
            .map_err(|e| WhisperError::tokenizer(e.to_string()))?;
        let avg_logprob = sum_logprob / tokens.len() as f64;

        Ok(Segment {
            start: 0.0,
            duration: 30.0,
            dr: DecodingResult {
                tokens,
                text,
                avg_logprob,
                no_speech_prob: f64::NAN, // Simplified - not calculating no_speech_prob
                temperature: temperature as f64,
                compression_ratio: f64::NAN, // Will be calculated by caller
            },
        })
    }

    /// Calculate a simple repetition ratio (total characters / unique characters).
    /// Higher values indicate more repetition.
    fn calculate_compression_ratio(&self, text: &str) -> f64 {
        if text.is_empty() {
            return 1.0;
        }
        
        let unique_chars: std::collections::HashSet<char> = text.chars().collect();
        let total_chars = text.chars().count();
        
        if total_chars == 0 {
            return 1.0;
        }
        
        total_chars as f64 / unique_chars.len() as f64
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