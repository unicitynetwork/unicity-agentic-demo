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
    dimensions: usize,
}

impl Embedding {
    const EXPECTED_DIMENSIONS: usize = 1_024;

    pub fn new() -> Result<Self, EmbeddingError> {
        let text_embedder = TextEmbedder::from_pretrained_hf(
            "Qwen3",
            "Qwen/Qwen3-Embedding-0.6B",
            None,
            None,
            Some(Dtype::F32),
        )
        .map_err(|e| EmbeddingError::ModelLoadError(e.to_string()))?;

        Ok(Self {
            text_embedder,
            dimensions: Self::EXPECTED_DIMENSIONS,
        })
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
        if embedding.len() != self.dimensions {
            return Err(EmbeddingError::DimensionMismatch {
                expected: self.dimensions,
                actual: embedding.len(),
            });
        }

        Ok(embedding)
    }

    /// Embed with late chunking (for long texts)
    pub async fn embed_batch(
        &self,
        texts: &[&str],
        batch_size: Option<usize>,
    ) -> EmbeddingServiceResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Err(EmbeddingError::InvalidInput(
                "Cannot embed empty batch".to_string(),
            ));
        }

        let results = self
            .text_embedder
            .embed(texts, batch_size, None)
            .await
            .map_err(|e| EmbeddingError::EmbeddingGenerationError(e.to_string()))?;

        // Extract all DenseVectors
        let mut embeddings = Vec::new();
        for result in results {
            match result {
                EmbeddingResult::DenseVector(vec) => {
                    if vec.len() != self.dimensions {
                        return Err(EmbeddingError::DimensionMismatch {
                            expected: self.dimensions,
                            actual: vec.len(),
                        });
                    }
                    embeddings.push(vec);
                }
                EmbeddingResult::MultiVector(_) => {
                    return Err(EmbeddingError::UnexpectedEmbeddingType);
                }
            }
        }

        Ok(embeddings)
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
}
