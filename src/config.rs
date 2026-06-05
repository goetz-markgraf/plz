#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub endpoint: String,
    pub api_key: String,
    #[serde(default)]
    pub model: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            api_key: String::new(),
            model: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        let home_dir = std::env::var("HOME")
                .map_err(|_| anyhow::anyhow!("Could not determine HOME directory"))?;
        let candidates = [
            Some(std::path::PathBuf::from(&home_dir).join(".config").join("plz").join("plz.json")),
            dirs::config_dir().map(|d| d.join("plz").join("plz.json")),
         ];

        let found = candidates.to_vec().into_iter().flatten().find(|p| p.exists());

        let config_path = found.ok_or_else(|| {
            let first = candidates[0].as_ref().unwrap();
            anyhow::anyhow!(
                 "Configuration file not found at {:?}\nPlease create it with your endpoint, api_key, and model.",
                first
             )
           })?;

        let content = std::fs::read_to_string(&config_path)
                .map_err(|e| anyhow::anyhow!("Failed to read configuration file: {}", e))?;
        let config: Self = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse configuration JSON: {}", e))?;
        Ok(config)
    }

    pub fn validate_for_query(&self) -> Result<(), anyhow::Error> {
        if self.endpoint.is_empty() {
            anyhow::bail!("Missing required field: endpoint in configuration");
        }
        if self.api_key.is_empty() {
            anyhow::bail!("Missing required field: api_key in configuration");
        }
        Ok(())
    }
}
