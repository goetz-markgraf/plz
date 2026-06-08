use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "role")]
    pub role: String,
    #[serde(rename = "content")]
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "messages")]
    pub messages: Vec<Message>,
    #[serde(rename = "stop", skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    #[serde(rename = "choices")]
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    #[serde(rename = "message")]
    pub message: Message,
}

#[derive(Debug)]
pub enum PlzError {
    InvalidApiKey(String),
    HttpError(u16, String),
    NetworkError(String),
    Anyhow(anyhow::Error),
}

impl std::fmt::Display for PlzError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlzError::InvalidApiKey(_msg) => {
                write!(
                    f,
                    "Invalid API key. Please check your configuration at ~/.config/plz/plz.json"
                )
            }
            PlzError::HttpError(status, msg) => {
                write!(f, "API error (HTTP {}): {}", status, msg)
            }
            PlzError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            PlzError::Anyhow(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for PlzError {}

pub struct PlzClient {
    endpoint: String,
    api_key: String,
    client: reqwest::Client,
}

impl PlzClient {
    pub fn new(endpoint: String, api_key: String) -> Self {
        Self {
            endpoint,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn chat_completion(
        &self,
        model: &str,
        system_prompt: &str,
        user_query: &str,
    ) -> Result<String, PlzError> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_query.to_string(),
            },
        ];

        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages,
            stop: None,
        };

        let url = format!("{}/chat/completions", self.endpoint);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| PlzError::NetworkError(format!("Failed to connect to {}: {}", url, e)))?;

        let status = response.status();

        if status.as_u16() == 401 {
            return Err(PlzError::InvalidApiKey(
                "The API key is invalid or expired".to_string(),
            ));
        }

        if status.is_client_error() || status.is_server_error() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(PlzError::HttpError(status.as_u16(), body));
        }

        let api_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| PlzError::Anyhow(anyhow::anyhow!("Failed to parse API response: {}", e)))?;

        let choices = api_response.choices;
        if choices.is_empty() {
            return Err(PlzError::Anyhow(anyhow::anyhow!(
                "API returned no choices"
            )));
        }

        Ok(choices[0].message.content.clone())
    }
}

impl From<anyhow::Error> for PlzError {
    fn from(e: anyhow::Error) -> Self {
        PlzError::Anyhow(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that special characters (dots, quotes, backslashes, Unicode) in
    /// the user query are correctly JSON-encoded and survive a round-trip
    /// through serde_json without corruption or URL-encoding artefacts.
    #[test]
    fn test_chat_request_serialises_special_chars() {
        let queries = &[
            r#"show me all files with .rs extension"#,
            r#"find "my file" in /tmp"#,
            r#"list files with \ backslash in name"#,
            "zeige Dateien mit Umlauten: ä ö ü",
        ];

        for query in queries {
            let req = ChatCompletionRequest {
                model: "test-model".to_string(),
                messages: vec![
                    Message { role: "user".to_string(), content: query.to_string() },
                ],
                stop: None,
            };
            let json = serde_json::to_string(&req)
                .expect("serialisation should not fail");
            let round_trip: serde_json::Value =
                serde_json::from_str(&json).expect("must be valid JSON");
            let content = round_trip["messages"][0]["content"]
                .as_str()
                .expect("content must be a string");
            assert_eq!(
                content, *query,
                "round-trip failed for query: {query}"
            );
            // Confirm no URL-percent-encoding crept in
            assert!(
                !content.contains('%'),
                "unexpected percent-encoding in serialised query: {content}"
            );
        }
    }
}
