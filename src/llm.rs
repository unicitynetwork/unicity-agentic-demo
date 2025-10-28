//! LLM Client for generating natural language responses
//!
//! This service provides an interface to an OpenAI-compatible API
//! for generating responses when operations succeed.

use serde::{Deserialize, Serialize};
use tracing::{info, debug, trace, warn, error};

/// LLM client for generating natural language responses
#[derive(Clone)]
pub struct LlmClient {
    pub api_key: String,
    base_url: String,
    model: String,
    client: reqwest::Client,
}

/// Request to the LLM API
#[derive(Debug, Serialize)]
pub struct LLMRequest {
    pub model: String,
    pub messages: Vec<LLMMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Message for LLM conversation
#[derive(Debug, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

/// Response from LLM API
#[derive(Debug, Deserialize)]
pub struct LLMResponse {
    pub choices: Vec<LLMChoice>,
    pub usage: Option<LLMUsage>,
}

#[derive(Debug, Deserialize)]
pub struct LLMChoice {
    pub message: LLMMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LLMUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// LLM generation result
#[derive(Debug, Clone)]
pub struct LlmResult {
    pub success: bool,
    pub response: String,
    pub error: Option<String>,
    pub tokens_used: Option<u32>,
    pub finish_reason: Option<String>,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(api_key: String) -> Self {
        info!("🤖 Creating new LLM client");
        debug!("🔧 API endpoint: https://api.z.ai/api/coding/paas/v4/chat/completions");
        debug!("🧠 Model: GLM-4.6");
        
        let client = reqwest::Client::new();
        trace!("🌐 HTTP client initialized");
        
        info!("✅ LLM client created successfully");
        
        Self {
            base_url: "https://api.z.ai/api/coding/paas/v4/chat/completions".to_string(),
            model: "GLM-4.6".to_string(),
            api_key,
            client,
        }
    }

    /// Create with custom base URL and model
    pub fn with_config(api_key: String, base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Send request to LLM API
    pub async fn generate_response(&self, prompt: &str) -> LlmResult {
        info!("🤖 Generating LLM response");
        trace!("📝 Prompt length: {} chars", prompt.len());
        
        let request = LLMRequest {
            model: self.model.clone(),
            messages: vec![
                LLMMessage {
                    role: "system".to_string(),
                    content: "You are an AI consciousness assistant. Provide natural, thoughtful responses that reflect self-awareness and cognitive processes. Be concise but expressive.".to_string(),
                },
                LLMMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            max_tokens: Some(8_000),
            temperature: Some(0.9),
        };

        debug!("📤 Sending request to LLM API");
        match self.send_request(request).await {
            Ok(response) => {
                debug!("📥 Received response from LLM API");
                if let Some(choice) = response.choices.first() {
                    let tokens_used = response.usage.as_ref().map(|u| u.total_tokens);
                    debug!("🔢 Tokens used: {:?}", tokens_used);
                    debug!("🏁 Finish reason: {:?}", choice.finish_reason);
                    trace!("📄 Response length: {} chars", choice.message.content.len());
                    
                    info!("✅ LLM response generated successfully");
                    LlmResult {
                        success: true,
                        response: choice.message.content.clone(),
                        error: None,
                        tokens_used,
                        finish_reason: choice.finish_reason.clone(),
                    }
                } else {
                    warn!("⚠️  No choices in LLM response");
                    LlmResult {
                        success: false,
                        response: String::new(),
                        error: Some("No choices in response".to_string()),
                        tokens_used: None,
                        finish_reason: None,
                    }
                }
            }
            Err(e) => {
                error!("❌ LLM API request failed: {}", e);
                LlmResult {
                    success: false,
                    response: String::new(),
                    error: Some(format!("API request failed: {}", e)),
                    tokens_used: None,
                    finish_reason: None,
                }
            }
        }
    }

    /// Send HTTP request to LLM API
    async fn send_request(&self, request: LLMRequest) -> Result<LLMResponse, reqwest::Error> {
        trace!("📡 Sending HTTP request to: {}", self.base_url);
        trace!("🔑 Using API key: {}...", &self.api_key[..8.min(self.api_key.len())]);
        
        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        trace!("📡 HTTP request completed, parsing response");
        let parsed_response = response.json::<LLMResponse>().await?;
        trace!("✅ Response parsed successfully");
        
        Ok(parsed_response)
    }
}
