use candle_transformers::models::whisper::{self as m, audio};
use rubato::{Resampler, FastFixedIn, PolynomialDegree};
use std::collections::VecDeque;
use candle_transformers::models::whisper::audio::pcm_to_mel;
use tracing::{debug, error};
use crate::error::{WhisperError, WhisperResult};
use crate::constants::{
    WHISPER_SAMPLE_RATE,
    AUDIO_CHUNK_SIZE,
    RESAMPLE_RATIO_MULTIPLIER,
    RESAMPLE_BUFFER_SIZE,
    NUM_FFT,
    MEL_FMIN,
    MEL_FMAX_MULTIPLIER,
    MEL_HZ_TO_MEL_CONST,
    MEL_HZ_TO_MEL_DIVISOR,
    MEL_MEL_TO_HZ_MULTIPLIER,
    MEL_MEL_TO_HZ_BASE,
    MEL_MEL_TO_HZ_DIVISOR
};

pub struct AudioProcessor {
    resampler: FastFixedIn<f32>,
    mel_filters: Vec<f32>,
    pub(crate) sample_rate: u32,
    pub(crate) buffer: VecDeque<f32>,
    pub(crate) chunk_size: usize,
    config: m::Config,
}

impl AudioProcessor {
    pub fn new(input_sample_rate: u32, config: &m::Config) -> WhisperResult<Self> {
        let resample_ratio = WHISPER_SAMPLE_RATE as f64 / input_sample_rate as f64;

        // Calculate chunk size at INPUT sample rate to get desired output
        // If we want 16000 samples at 16kHz output, we need:
        // chunk_size_input = 16000 / resample_ratio
        // For 48kHz->16kHz: 16000 / 0.333 = 48000 samples
        let chunk_size_at_input_rate = (AUDIO_CHUNK_SIZE as f64 / resample_ratio) as usize;

        println!("🔍 AudioProcessor config:");
        println!("   Input rate: {} Hz", input_sample_rate);
        println!("   Output rate: {} Hz", WHISPER_SAMPLE_RATE);
        println!("   Resample ratio: {:.3}", resample_ratio);
        println!("   Chunk size at input rate: {} samples", chunk_size_at_input_rate);
        println!("   Expected output: {} samples", AUDIO_CHUNK_SIZE);

        let resampler = FastFixedIn::new(
            resample_ratio,
            RESAMPLE_RATIO_MULTIPLIER,
            PolynomialDegree::Septic,
            RESAMPLE_BUFFER_SIZE,
            1,
        )?;

        let mel_filters = generate_mel_filters(config.num_mel_bins)?;

        Ok(Self {
            resampler,
            mel_filters,
            sample_rate: input_sample_rate,
            buffer: VecDeque::new(),
            chunk_size: chunk_size_at_input_rate,  // ← Store input chunk size!
            config: config.clone(),
        })
    }

    pub fn process_audio(&mut self, audio: &[f32]) -> WhisperResult<Vec<Vec<f32>>> {
        self.buffer.extend(audio);
        let mut chunks = Vec::new();

        // Process complete chunks
        while self.buffer.len() >= self.chunk_size {
            // Take exactly chunk_size samples
            let samples_to_resample: Vec<f32> = self.buffer.drain(..self.chunk_size).collect();

            // Resample in 1024-sample pieces
            let full_chunks = samples_to_resample.len() / RESAMPLE_BUFFER_SIZE;
            let mut resampled_chunk = Vec::new();

            for i in 0..full_chunks {
                let start = i * RESAMPLE_BUFFER_SIZE;
                let end = start + RESAMPLE_BUFFER_SIZE;
                let chunk = &samples_to_resample[start..end];
                let resampled = self.resampler.process(&[chunk], None)?;
                resampled_chunk.extend_from_slice(&resampled[0]);
            }

            println!("🔧 Processed chunk: {} input samples -> {} output samples",
                     samples_to_resample.len(), resampled_chunk.len());

            chunks.push(resampled_chunk);
        }

        Ok(chunks)
    }

    pub fn flush(&mut self) -> WhisperResult<Option<Vec<f32>>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }
        
        let remaining: Vec<f32> = self.buffer.drain(..).collect();
        
        // Resample if needed
        let resampled = if self.sample_rate != WHISPER_SAMPLE_RATE {
            let pcm_vec = vec![remaining];
            let resampled = self.resampler.process(&pcm_vec, None)?;
            resampled.into_iter().flatten().collect()
        } else {
            remaining
        };
        
        Ok(Some(resampled))
    }

    pub fn has_enough_audio(&self) -> bool {
        self.buffer.len() >= self.chunk_size
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}

// Voice Activity Detection (VAD) to avoid processing silence
pub struct VoiceActivityDetector {
    threshold: f32,
    window_size: usize,
    energy_history: VecDeque<f32>,
}

impl VoiceActivityDetector {
    pub fn new(threshold: f32, window_size: usize) -> Self {
        Self {
            threshold,
            window_size,
            energy_history: VecDeque::with_capacity(window_size),
        }
    }

    pub fn is_speech(&mut self, audio: &[f32]) -> bool {
        if audio.is_empty() {
            return false;
        }

        // Calculate energy
        let energy: f32 = audio.iter().map(|&x| x * x).sum::<f32>() / audio.len() as f32;
        
        // Add to history
        self.energy_history.push_back(energy);
        if self.energy_history.len() > self.window_size {
            self.energy_history.pop_front();
        }

        // Average energy over window
        let avg_energy = self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32;
        
        avg_energy > self.threshold
    }

    pub fn reset(&mut self) {
        self.energy_history.clear();
    }
}

pub fn generate_mel_filters(num_mel_bins: usize) -> WhisperResult<Vec<f32>> {
    let sample_rate = WHISPER_SAMPLE_RATE;
    let fmin = MEL_FMIN;
    let fmax = sample_rate as f64 * MEL_FMAX_MULTIPLIER;

    // Convert to mel scale
    fn hz_to_mel(hz: f64) -> f64 {
        MEL_HZ_TO_MEL_CONST * (1.0 + hz / MEL_HZ_TO_MEL_DIVISOR).log10()
    }

    fn mel_to_hz(mel: f64) -> f64 {
        MEL_MEL_TO_HZ_MULTIPLIER * (MEL_MEL_TO_HZ_BASE.powf(mel / MEL_MEL_TO_HZ_DIVISOR) - 1.0)
    }

    // Create mel frequency points
    let mut mel_points = Vec::with_capacity(num_mel_bins + 2);
    for i in 0..=num_mel_bins + 1 {
        let mel = hz_to_mel(fmin) + (hz_to_mel(fmax) - hz_to_mel(fmin)) * i as f64 / (num_mel_bins + 1) as f64;
        mel_points.push(mel);
    }
    
    // Convert back to Hz
    let hz_points: Vec<f64> = mel_points.iter().map(|&mel| mel_to_hz(mel)).collect();
    
    // Create FFT bin points
    let bin_points: Vec<usize> = hz_points.iter()
        .map(|&hz| (hz * NUM_FFT as f64 / 2.0 / sample_rate as f64).floor() as usize)
        .collect();
    
    // Create filter bank
    let mut filters = vec![0.0f32; num_mel_bins * (NUM_FFT / 2 + 1)];
    
    for i in 1..=num_mel_bins {
        let left = bin_points[i - 1];
        let center = bin_points[i];
        let right = bin_points[i + 1];
        
        // Create triangle filter
        for j in left..=center {
            if j < NUM_FFT / 2 + 1 {
                filters[(i - 1) * (NUM_FFT / 2 + 1) + j] = ((j - left) as f32) / ((center - left) as f32);
            }
        }
        for j in center..=right {
            if j < NUM_FFT / 2 + 1 {
                filters[(i - 1) * (NUM_FFT / 2 + 1) + j] = ((right - j) as f32) / ((right - center) as f32);
            }
        }
    }
    
    Ok(filters)
}

#[cfg(test)]
mod tests {
    use crate::constants::{VAD_THRESHOLD, VAD_WINDOW_SIZE};
    use super::*;

    #[test]
    fn test_vad() {
        let mut vad = VoiceActivityDetector::new(VAD_THRESHOLD, VAD_WINDOW_SIZE);
        
        // Silence
        let silence = vec![0.0; 1000];
        assert!(!vad.is_speech(&silence));
        
        // Speech (simulated with higher energy)
        let speech = vec![0.1; 1000];
        assert!(vad.is_speech(&speech));
    }
}