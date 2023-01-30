use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub description: String,
}
