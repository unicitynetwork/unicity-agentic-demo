use embed_anything::Dtype;
use embed_anything::embeddings::embed::{EmbeddingResult, TextEmbedder};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),

    #[error("Failed to generate embedding: {0}")]
    EmbeddingGenerationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unexpected embedding type: expected DenseVector, got MultiVector")]
    UnexpectedEmbeddingType,

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

pub type EmbeddingServiceResult<T> = Result<T, EmbeddingError>;

pub struct Embedding {
    text_embedder: TextEmbedder,
}

impl Embedding {
    const EXPECTED_DIMENSIONS: usize = 1_024;

    pub fn new() -> Result<Self, EmbeddingError> {
        let text_embedder = TextEmbedder::from_pretrained_hf(
            "Qwen3",
            "Qwen/Qwen3-Embedding-0.6B",
            None,
            None,
            Some(Dtype::BF16),
        ).map_err(|e| EmbeddingError::ModelLoadError(e.to_string()))?;

        Ok(Self { text_embedder })
    }

    pub async fn embed(&self, text: &str) -> EmbeddingServiceResult<Vec<f32>> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput(
                "Cannot embed empty text".to_string(),
            ));
        }

        let results = self
            .text_embedder
            .embed(
                &[text],
                None, // batch_size
                None, // late_chunking
            )
            .await
            .map_err(|e| EmbeddingError::EmbeddingGenerationError(e.to_string()))?;

        // Extract DenseVector from EmbeddingResult enum
        let embedding = match results.first() {
            Some(EmbeddingResult::DenseVector(vec)) => vec.clone(),
            Some(EmbeddingResult::MultiVector(_)) => {
                return Err(EmbeddingError::UnexpectedEmbeddingType);
            }
            None => {
                return Err(EmbeddingError::EmbeddingGenerationError(
                    "No embedding returned".to_string(),
                ));
            }
        };

        // Verify dimensions
        if embedding.len() != Self::EXPECTED_DIMENSIONS {
            return Err(EmbeddingError::DimensionMismatch {
                expected: Self::EXPECTED_DIMENSIONS,
                actual: embedding.len(),
            });
        }

        Ok(embedding)
    }
}
