use crate::models::{Database, Provider, Model, Cost, Limit, Modalities, Modality, Interleaved, InterleavedField, ProviderInfo, ModelStatus};
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use dashmap::DashMap;
use rayon::prelude::*;
use tracing::{info, warn, debug};
use regex::Regex;
use chrono::{DateTime, Utc, NaiveDate};

lazy_static::lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}(-\d{2})?$").unwrap();
    static ref KNOWLEDGE_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}(-\d{2})?$").unwrap();
}

pub struct Parser {
    cache: Arc<DashMap<String, Arc<Provider>>>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn parse_directory(&self, providers_dir: &Path) -> Result<Database> {
        info!("Parsing providers directory: {:?}", providers_dir);
        
        if !providers_dir.exists() {
            return Err(Error::DirectoryNotFound(providers_dir.to_path_buf()));
        }

        let entries = fs::read_dir(providers_dir)
            .map_err(|e| Error::IoError(format!("Failed to read providers directory: {}", e)))?;

        let provider_paths: Vec<_> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_dir() {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        info!("Found {} provider directories", provider_paths.len());

        let mut database = Database::new();
        
        // Parse providers in parallel
        let providers: Result<Vec<_>> = provider_paths
            .par_iter()
            .map(|provider_path| self.parse_provider(provider_path))
            .collect();

        let providers = providers?;
        
        for provider in providers {
            info!("Loaded provider: {} with {} models", provider.name, provider.models.len());
            database.add_provider(provider);
        }

        info!("Successfully parsed {} providers with {} models", 
              database.provider_count(), database.model_count());

        Ok(database)
    }

    fn parse_provider(&self, provider_dir: &Path) -> Result<Provider> {
        let provider_id = provider_dir.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::InvalidPath("Invalid provider directory name".to_string()))?;

        // Check cache first
        if let Some(cached_provider) = self.cache.get(provider_id) {
            debug!("Using cached provider: {}", provider_id);
            return Ok((*cached_provider).as_ref().clone());
        }

        let provider_toml_path = provider_dir.join("provider.toml");
        if !provider_toml_path.exists() {
            return Err(Error::FileNotFound(provider_toml_path));
        }

        let content = fs::read_to_string(&provider_toml_path)
            .map_err(|e| Error::IoError(format!("Failed to read provider.toml: {}", e)))?;

        let mut provider: ProviderData = toml::from_str(&content)
            .map_err(|e| Error::TomlParse(format!("Failed to parse provider.toml: {}", e)))?;

        // Validate provider
        self.validate_provider(&provider)?;

        // Parse models
        let models_dir = provider_dir.join("models");
        let models = if models_dir.exists() {
            self.parse_models(&models_dir)?
        } else {
            HashMap::new()
        };

        let provider_obj = Provider {
            id: provider_id.to_string(),
            name: provider.name,
            npm: provider.npm,
            env: provider.env,
            api: provider.api,
            doc: provider.doc,
            models,
        };

        // Cache the provider
        self.cache.insert(provider_id.to_string(), Arc::new(provider_obj.clone()));

        Ok(provider_obj)
    }

    fn parse_models(&self, models_dir: &Path) -> Result<HashMap<String, Model>> {
        let entries = fs::read_dir(models_dir)
            .map_err(|e| Error::IoError(format!("Failed to read models directory: {}", e)))?;

        let model_paths: Vec<_> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // Parse models in parallel
        let models: Result<Vec<_>> = model_paths
            .par_iter()
            .map(|model_path| self.parse_model(model_path))
            .collect();

        let models = models?;
        
        let mut model_map = HashMap::new();
        for model in models {
            model_map.insert(model.id.clone(), model);
        }

        Ok(model_map)
    }

    fn parse_model(&self, model_path: &Path) -> Result<Model> {
        let model_id = model_path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::InvalidPath("Invalid model file name".to_string()))?;

        // Handle nested model IDs (e.g., openai/gpt-4.toml)
        let relative_path = model_path.strip_prefix(model_path.parent().unwrap().parent().unwrap())
            .map_err(|_| Error::InvalidPath("Failed to get relative path".to_string()))?;
        let full_model_id = relative_path
            .with_extension("")
            .to_str()
            .ok_or_else(|| Error::InvalidPath("Invalid model path".to_string()))?
            .replace(std::path::MAIN_SEPARATOR_STR, "/");

        let content = fs::read_to_string(model_path)
            .map_err(|e| Error::IoError(format!("Failed to read model file: {}", e)))?;

        let mut model_data: ModelData = toml::from_str(&content)
            .map_err(|e| Error::TomlParse(format!("Failed to parse model.toml: {}", e)))?;

        // Set the model ID
        model_data.id = full_model_id.to_string();

        // Validate model
        self.validate_model(&model_data)?;

        let model = Model {
            id: model_data.id,
            name: model_data.name,
            family: model_data.family,
            attachment: model_data.attachment,
            reasoning: model_data.reasoning,
            tool_call: model_data.tool_call,
            interleaved: model_data.interleaved.map(|i| match i {
                InterleavedData::Boolean(b) => Interleaved::Boolean(b),
                InterleavedData::Object { field } => Interleaved::Object { field },
            }),
            structured_output: model_data.structured_output,
            temperature: model_data.temperature,
            knowledge: model_data.knowledge,
            release_date: model_data.release_date,
            last_updated: model_data.last_updated,
            modalities: model_data.modalities,
            open_weights: model_data.open_weights,
            cost: model_data.cost,
            limit: model_data.limit,
            status: model_data.status,
            provider: model_data.provider,
        };

        Ok(model)
    }

    fn validate_provider(&self, provider: &ProviderData) -> Result<()> {
        if provider.name.is_empty() {
            return Err(Error::Validation("Provider name cannot be empty".to_string()));
        }

        if provider.npm.is_empty() {
            return Err(Error::Validation("Provider npm cannot be empty".to_string()));
        }

        if provider.env.is_empty() {
            return Err(Error::Validation("Provider env cannot be empty".to_string()));
        }

        if provider.doc.is_empty() {
            return Err(Error::Validation("Provider documentation URL is required".to_string()));
        }

        // Validate API field based on npm package
        let is_openai_compatible = provider.npm == "@ai-sdk/openai-compatible";
        let is_openrouter = provider.npm == "@openrouter/ai-sdk-provider";
        
        if (is_openai_compatible || is_openrouter) && provider.api.is_none() {
            return Err(Error::Validation(
                "API endpoint is required for openai-compatible and openrouter providers".to_string()
            ));
        }

        if !is_openai_compatible && !is_openrouter && provider.api.is_some() {
            return Err(Error::Validation(
                "API endpoint is only allowed for openai-compatible and openrouter providers".to_string()
            ));
        }

        Ok(())
    }

    fn validate_model(&self, model: &ModelData) -> Result<()> {
        if model.name.is_empty() {
            return Err(Error::Validation("Model name cannot be empty".to_string()));
        }

        if !DATE_REGEX.is_match(&model.release_date) {
            return Err(Error::Validation("Release date must be in YYYY-MM or YYYY-MM-DD format".to_string()));
        }

        if !DATE_REGEX.is_match(&model.last_updated) {
            return Err(Error::Validation("Last updated date must be in YYYY-MM or YYYY-MM-DD format".to_string()));
        }

        if let Some(ref knowledge) = model.knowledge {
            if !KNOWLEDGE_REGEX.is_match(knowledge) {
                return Err(Error::Validation("Knowledge date must be in YYYY-MM or YYYY-MM-DD format".to_string()));
            }
        }

        // Validate cost
        if let Some(ref cost) = model.cost {
            if cost.input < 0.0 {
                return Err(Error::Validation("Input cost cannot be negative".to_string()));
            }
            if cost.output < 0.0 {
                return Err(Error::Validation("Output cost cannot be negative".to_string()));
            }
            if let Some(reasoning) = cost.reasoning {
                if reasoning < 0.0 {
                    return Err(Error::Validation("Reasoning cost cannot be negative".to_string()));
                }
            }
            if let Some(cache_read) = cost.cache_read {
                if cache_read < 0.0 {
                    return Err(Error::Validation("Cache read cost cannot be negative".to_string()));
                }
            }
            if let Some(cache_write) = cost.cache_write {
                if cache_write < 0.0 {
                    return Err(Error::Validation("Cache write cost cannot be negative".to_string()));
                }
            }
        }

        // Validate limits
        if model.limit.context == 0 {
            return Err(Error::Validation("Context limit must be positive".to_string()));
        }
        if model.limit.output == 0 {
            return Err(Error::Validation("Output limit must be positive".to_string()));
        }
        if let Some(input) = model.limit.input {
            if input == 0 {
                return Err(Error::Validation("Input limit must be positive".to_string()));
            }
        }

        // Validate reasoning cost consistency
        if !model.reasoning && model.cost.as_ref().and_then(|c| c.reasoning).is_some() {
            return Err(Error::Validation("Cannot set cost.reasoning when reasoning is false".to_string()));
        }

        Ok(())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

// Internal data structures for TOML parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderData {
    name: String,
    npm: String,
    env: Vec<String>,
    api: Option<String>,
    doc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelData {
    #[serde(default)]
    id: String,
    name: String,
    family: Option<String>,
    attachment: bool,
    reasoning: bool,
    tool_call: bool,
    interleaved: Option<InterleavedData>,
    structured_output: Option<bool>,
    temperature: Option<bool>,
    knowledge: Option<String>,
    release_date: String,
    last_updated: String,
    modalities: Modalities,
    open_weights: bool,
    cost: Option<Cost>,
    limit: Limit,
    status: Option<ModelStatus>,
    provider: Option<ProviderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum InterleavedData {
    Boolean(bool),
    Object { field: InterleavedField },
}
