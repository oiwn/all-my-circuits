use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub openai_api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai_api_key: "YOUR_API_KEY".to_string(),
        }
    }
}
