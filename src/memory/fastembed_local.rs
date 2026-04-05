//! Local ONNX embeddings via `fastembed` (`plugin-fastembed`).

use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use super::embeddings::EmbeddingProvider;

pub struct FastembedEmbedding {
    model: Arc<Mutex<TextEmbedding>>,
    dimensions: usize,
}

impl FastembedEmbedding {
    pub fn new(model: Option<String>) -> Result<Self> {
        let model_name = match model.unwrap_or_default().to_ascii_lowercase().as_str() {
            "large" | "all-minilm-l12" | "all_mini_lm_l12" => EmbeddingModel::AllMiniLML12V2,
            _ => EmbeddingModel::AllMiniLML6V2,
        };
        let model =
            TextEmbedding::try_new(InitOptions::new(model_name).with_show_download_progress(false))
                .context("fastembed TextEmbedding::try_new")?;
        let sample = model
            .embed(vec!["ping"], None)
            .context("fastembed probe embed")?;
        let dimensions = sample.first().map(|v| v.len()).unwrap_or(384);
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            dimensions,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for FastembedEmbedding {
    fn name(&self) -> &str {
        "fastembed"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let owned: Vec<String> = texts.iter().map(|s| (*s).to_string()).collect();
        let model = Arc::clone(&self.model);
        tokio::task::spawn_blocking(move || {
            let m = model
                .lock()
                .map_err(|_| anyhow::anyhow!("fastembed model mutex poisoned"))?;
            m.embed(owned, None).context("fastembed embed")
        })
        .await
        .context("fastembed join")?
    }
}
