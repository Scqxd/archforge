//! Chutes API client for MiniMaxAI/MiniMax-M2.5-TEE
//!
//! Optimized with:
//! - Connection pooling via lazy_static Client
//! - Response caching to avoid redundant API calls
//! - Efficient spinner animation with early termination

use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::RwLock;

/// Spinner frames for animation
const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Global HTTP client with connection pooling (lazy initialization)
static HTTP_CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

/// Response cache for AI generations (avoids redundant API calls)
static RESPONSE_CACHE: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

/// Get or create the global HTTP client with optimized settings
fn get_http_client() -> &'static reqwest::blocking::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(30))
            .timeout(Duration::from_secs(600))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    })
}

/// Get or create the response cache
fn get_response_cache() -> &'static RwLock<HashMap<String, String>> {
    RESPONSE_CACHE.get_or_init(|| RwLock::new(HashMap::with_capacity(16)))
}

/// Chutes API client
#[derive(Debug, Clone)]
pub struct ChutesClient {
    api_key: String,
    base_url: String,
    model: String,
    #[allow(dead_code)] // Kept for future per-request timeout customization
    timeout_secs: u64,
}

impl ChutesClient {
    /// Create a new ChutesClient with default timeout (600s - 10 minutes)
    pub fn new(api_key: String) -> Self {
        Self::with_timeout(api_key, 600)
    }

    /// Create a new ChutesClient with custom timeout
    pub fn with_timeout(api_key: String, timeout_secs: u64) -> Self {
        Self {
            api_key,
            base_url: "https://llm.chutes.ai/v1".to_string(),
            model: "MiniMaxAI/MiniMax-M2.5-TEE".to_string(),
            timeout_secs,
        }
    }

    /// Generate a PKGBUILD from a description
    pub fn generate_pkgbuild(&self, description: &str) -> Result<String, Box<dyn Error>> {
        // Check cache first (optimization: avoid redundant API calls)
        let cache_key = format!("{}:{}", self.model, description);
        if let Some(cached) = get_response_cache().read().unwrap().get(&cache_key) {
            eprintln!("[AI] Cache hit for description");
            return Ok(cached.clone());
        }

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

        // Use global client with connection pooling (optimization: reuse connections)
        let client = get_http_client();

        // Start spinner animation
        let stop_spinner = Arc::new(AtomicBool::new(false));
        let stop_spinner_clone = Arc::clone(&stop_spinner);

        let spinner_handle = thread::spawn(move || {
            let mut stdout = io::stdout();
            let mut frame_idx = 0;

            // Save cursor position and hide cursor
            print!("\x1b[?25l"); // Hide cursor
            let _ = stdout.flush();

            while !stop_spinner_clone.load(Ordering::Relaxed) {
                let frame = SPINNER_FRAMES[frame_idx % SPINNER_FRAMES.len()];
                print!("\r\x1b[K{} Generation...", frame); // \x1b[K clears the line
                let _ = stdout.flush();
                frame_idx += 1;
                thread::sleep(Duration::from_millis(80));
            }

            // Restore cursor
            print!("\r\x1b[?25h\x1b[K"); // Show cursor and clear line
            let _ = stdout.flush();
        });

        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send();

        // Stop spinner immediately on response (optimization: faster UI feedback)
        stop_spinner.store(true, Ordering::Relaxed);
        let _ = spinner_handle.join();

        let response = response?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Chutes API error ({}): {}", status, error_text).into());
        }

        let response_json: ChutesResponse = response.json()?;

        if let Some(text) = response_json.choices.first() {
            let mut content = text.message.content.clone();
            eprintln!("[AI] Response received ({} chars)", content.len());

            // Extract PKGBUILD from response (handle markdown code blocks)
            content = extract_pkgbuild_from_markdown(&content);

            if content.starts_with("ERROR") {
                return Err(format!("AI generation failed: {}", content).into());
            }

            // Ensure it's a valid PKGBUILD
            if content.contains("pkgname=") && content.contains("build()") {
                // Cache the result for future requests (optimization: avoid redundant API calls)
                if let Ok(mut cache) = get_response_cache().write() {
                    cache.insert(cache_key, content.clone());
                }
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

#[derive(Deserialize, Debug, Serialize)]
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

#[derive(Deserialize, Debug, Serialize)]
struct ChutesChoice {
    #[allow(dead_code)]
    index: u32,
    message: ChutesMessage,
    #[allow(dead_code)]
    finish_reason: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct ChutesMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
}

/// Clear the response cache (for cache management commands)
pub fn clear_response_cache() {
    if let Ok(mut cache) = get_response_cache().write() {
        cache.clear();
        eprintln!("[AI] Response cache cleared");
    }
}

/// Get the number of cached responses (for stats)
pub fn get_response_cache_for_stats() -> usize {
    get_response_cache().read().map(|c| c.len()).unwrap_or(0)
}
