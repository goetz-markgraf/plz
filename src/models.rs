use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Model {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "object")]
    pub object_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

pub async fn list_models(endpoint: &str, api_key: &str) -> Result<Vec<Model>, String> {
    let url = format!("{}/models", endpoint);

    let response = reqwest::Client::new()
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to connect to {}: {}", url, e))?;

    let status = response.status();

    if status.as_u16() == 401 {
        return Err("Invalid API key. Please check your configuration at ~/.config/plz/plz.json".to_string());
    }

    if status.is_client_error() || status.is_server_error() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        return Err(format!("API error (HTTP {}): {}", status.as_u16(), body));
    }

    let models_response: ModelsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse models response: {}", e))?;

    Ok(models_response.data)
}

pub fn format_models(models: &[Model]) {
    println!("Available models:\n");
    for model in models {
        let display_name = model.name.as_deref().unwrap_or(&model.id);
        println!("  {} ({}) [type: {}]", model.id, display_name, model.object_type);
    }
    println!(
        "\nPlease configure a model in your configuration file at ~/.config/plz/plz.json"
    );
}
