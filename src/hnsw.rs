use hnsw_rs::hnsw::Hnsw;
use hnsw_rs::prelude::DistDot;
use std::collections::HashMap;
use surrealdb::RecordId;
use thiserror::Error;

// https://arxiv.org/abs/1603.09320
// https://opensearch.org/blog/a-practical-guide-to-selecting-hnsw-hyperparameters/
// https://milvus.io/ai-quick-reference/what-are-the-key-configuration-parameters-for-an-hnsw-index-such-as-m-and-efconstructionefsearch-and-how-does-each-influence-the-tradeoff-between-index-size-build-time-query-speed-and-recall

#[derive(Error, Debug)]
pub enum HnswError {
    #[error("HNSW index needs rebuild: {current}/{max} items")]
    NeedsRebuild { current: usize, max: usize },
}

pub struct HnswMemoryIndex<'a> {
    hnsw: Hnsw<'a, f32, DistDot>,
    id_to_memory: HashMap<usize, RecordId>,
    current_size: usize,
    max_capacity: usize,
    dimensions: usize,
}

impl<'a> HnswMemoryIndex<'a> {
    /// Create a new HNSW index with specified capacity (including buffer)
    ///
    /// # Arguments
    /// * `capacity` - Total capacity including buffer (e.g., 100 memories + buffer = 500)
    /// * `dimensions` - Embedding dimensionality (e.g., 1024 for Qwen3-0.6B)
    ///
    /// # Example
    /// ```
    /// // You have 100 memories, want 5x buffer
    /// let index = HnswMemoryIndex::new(500, 1024);
    /// ```
    pub fn new(capacity: usize, dimensions: usize) -> Self {
        // Calculate optimal parameters based on capacity and dimensions
        let m = Self::calculate_m(dimensions);
        let max_layer = Self::calculate_max_layer(capacity);
        let ef_construction = Self::calculate_ef_construction(m);

        let hnsw = Hnsw::<f32, DistDot>::new(m, capacity, max_layer, ef_construction, DistDot);

        Self {
            hnsw,
            id_to_memory: HashMap::new(),
            current_size: 0,
            max_capacity: capacity,
            dimensions,
        }
    }

    /// Add a single memory (relation) to the index
    pub fn add(&mut self, id: RecordId, embedding: &[f32]) -> Result<(), HnswError> {
        if self.current_size >= self.max_capacity {
            return Err(HnswError::NeedsRebuild {
                current: self.current_size,
                max: self.max_capacity,
            });
        }

        let hnsw_id = self.current_size;
        self.hnsw.insert((embedding, hnsw_id));
        self.id_to_memory.insert(hnsw_id, id);
        self.current_size += 1;

        Ok(())
    }

    /// Add multiple memories in a batch (more efficient)
    pub fn add_relations(&mut self, memories: Vec<(RecordId, &[f32])>) -> Result<(), HnswError> {
        for (id, embedding) in memories {
            self.add(id, embedding)?;
        }
        Ok(())
    }

    /// Search for top K nearest neighbors
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(RecordId, f32)> {
        let m = Self::calculate_m(self.dimensions);
        let ef_search = (m * 4).max(100); // 4x M, minimum 100

        let neighbors = self.hnsw.search(query_embedding, top_k, ef_search);

        neighbors
            .iter()
            .map(|neighbor| {
                let memory_id = self.id_to_memory.get(&neighbor.d_id).unwrap().clone();
                let similarity = 1.0 - neighbor.distance;
                (memory_id, similarity)
            })
            .collect()
    }

    /// Check if index is approaching capacity (>85% full)
    pub fn needs_rebuild(&self) -> bool {
        self.usage_percent() >= 85.0
    }

    /// Get current usage as percentage
    pub fn usage_percent(&self) -> f32 {
        (self.current_size as f32 / self.max_capacity as f32) * 100.0
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.current_size
    }

    /// Get max capacity
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.current_size == 0
    }

    /// Calculate optimal M based on dimensionality
    fn calculate_m(dimensions: usize) -> usize {
        match dimensions {
            0..=128 => 12,
            129..=512 => 16,
            513..=1024 => 24,
            _ => 32,
        }
    }

    /// Calculate max_layer: log₂(capacity)
    fn calculate_max_layer(capacity: usize) -> usize {
        (capacity as f32).log2().ceil() as usize
    }

    /// Calculate ef_construction: 12x M
    fn calculate_ef_construction(m: usize) -> usize {
        (m * 12).clamp(100, 400)
    }

    // TODO: Store and Load.
}
