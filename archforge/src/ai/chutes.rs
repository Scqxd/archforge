//! Chutes API client for MiniMaxAI/MiniMax-M2.1-TEE

use reqwest;
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;

/// Chutes API client
#[derive(Debug, Clone)]
pub struct ChutesClient {
    api_key: String,
    base_url: String,
    model: String,
    timeout_secs: u64,
}

impl ChutesClient {
    /// Create a new ChutesClient with default timeout (30s)
    pub fn new(api_key: String) -> Self {
        Self::with_timeout(api_key, 30)
    }

    /// Create a new ChutesClient with custom timeout
    pub fn with_timeout(api_key: String, timeout_secs: u64) -> Self {
        Self {
            api_key,
            base_url: "https://api.chutes.ai/v1".to_string(),
            model: "MiniMaxAI/MiniMax-M2.1-TEE".to_string(),
            timeout_secs,
        }
    }

    /// Generate a PKGBUILD from a description
    pub fn generate_pkgbuild(&self, description: &str) -> Result<String, Box<dyn Error>> {
        eprintln!("[AI] Calling Chutes API (MiniMaxAI/MiniMax-M2.1-TEE)...");

        let prompt = format!(
            r#"Generate a complete, valid Arch Linux PKGBUILD for the following package description:

"{}"

Requirements:
- Follow Arch Linux packaging standards
- Use appropriate build() and package() functions
- Include all necessary dependencies (depends, makedepends, optdepends)
- Set proper pkgname, pkgver, pkgrel, pkgdesc, arch, url, license
- Use sha256sums=('SKIP') for source URLs
- Add helpful comments where needed

Return ONLY the PKGBUILD content, no markdown, no explanations. If you cannot generate a valid PKGBUILD, return only the word "ERROR" followed by a brief reason."#,
            description
        );

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": 4096,
            "temperature": 0.7
        });

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()?;

        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text()?;
            return Err(format!("Chutes API error ({}): {}", status, error_text).into());
        }

        let response_json: ChutesResponse = response.json()?;

        if let Some(text) = response_json.choices.first() {
            let content = text.message.content.clone();
            eprintln!("[AI] Response received ({} chars)", content.len());

            // Extract PKGBUILD from response (handle markdown code blocks)
            let content = extract_pkgbuild_from_markdown(&content);

            if content.starts_with("ERROR") {
                return Err(format!("AI generation failed: {}", content).into());
            }

            // Ensure it's a valid PKGBUILD
            if content.contains("pkgname=") && content.contains("build()") {
                Ok(content)
            } else {
                Err("AI response does not contain a valid PKGBUILD".into())
            }
        } else {
            Err("No response from AI model".into())
        }
    }
}

/// Extract PKGBUILD content from markdown code blocks
fn extract_pkgbuild_from_markdown(content: &str) -> String {
    let trimmed = content.trim();
    
    // Try to strip markdown code block with language
    let stripped = trimmed
        .strip_prefix("```bash")
        .or_else(|| trimmed.strip_prefix("```sh"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    
    // Remove trailing code block marker and trim
    stripped
        .trim_end()
        .strip_suffix("```")
        .unwrap_or(stripped)
        .trim()
        .to_string()
}

#[derive(Deserialize, Debug)]
struct ChutesResponse {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    created: u64,
    #[allow(dead_code)]
    model: String,
    choices: Vec<ChutesChoice>,
}

#[derive(Deserialize, Debug)]
struct ChutesChoice {
    #[allow(dead_code)]
    index: u32,
    message: ChutesMessage,
    #[allow(dead_code)]
    finish_reason: String,
}

#[derive(Deserialize, Debug)]
struct ChutesMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
}