use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
    Pdf,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Modalities {
    pub input: Vec<Modality>,
    pub output: Vec<Modality>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Cost {
    pub input: f64,
    pub output: f64,
    pub reasoning: Option<f64>,
    pub cache_read: Option<f64>,
    pub cache_write: Option<f64>,
    pub input_audio: Option<f64>,
    pub output_audio: Option<f64>,
    #[serde(rename = "context_over_200k")]
    pub context_over_200k: Option<Box<Cost>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Limit {
    pub context: usize,
    pub input: Option<usize>,
    pub output: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    Alpha,
    Beta,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InterleavedField {
    ReasoningContent,
    ReasoningDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Interleaved {
    Boolean(bool),
    Object { field: InterleavedField },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProviderInfo {
    pub npm: Option<String>,
    pub api: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub family: Option<String>,
    pub attachment: bool,
    pub reasoning: bool,
    pub tool_call: bool,
    pub interleaved: Option<Interleaved>,
    pub structured_output: Option<bool>,
    pub temperature: Option<bool>,
    pub knowledge: Option<String>,
    pub release_date: String,
    pub last_updated: String,
    pub modalities: Modalities,
    pub open_weights: bool,
    pub cost: Option<Cost>,
    pub limit: Limit,
    pub status: Option<ModelStatus>,
    pub provider: Option<ProviderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub npm: String,
    pub env: Vec<String>,
    pub api: Option<String>,
    pub doc: String,
    pub models: HashMap<String, Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Database {
    pub providers: HashMap<String, Provider>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Provider) {
        self.providers.insert(provider.id.clone(), provider);
    }

    pub fn get_provider(&self, id: &str) -> Option<&Provider> {
        self.providers.get(id)
    }

    pub fn get_model(&self, provider_id: &str, model_id: &str) -> Option<&Model> {
        self.providers.get(provider_id)?.models.get(model_id)
    }

    pub fn all_models(&self) -> impl Iterator<Item = (&String, &String, &Model)> {
        self.providers.iter().flat_map(|(provider_id, provider)| {
            provider.models.iter().map(move |(model_id, model)| (provider_id, model_id, model))
        })
    }

    pub fn model_count(&self) -> usize {
        self.providers.values().map(|p| p.models.len()).sum()
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
