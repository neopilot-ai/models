use crate::models::Database;
use crate::parser::Parser;
use crate::error::Result;
use std::path::Path;
use std::sync::Arc;
use dashmap::DashMap;
use serde_json::Value;
use schemars::schema::RootSchema;
use schemars::gen::{SchemaGenerator, SchemaSettings};

pub struct Engine {
    database: Arc<Database>,
    parser: Arc<Parser>,
    cache: Arc<DashMap<String, Value>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            database: Arc::new(Database::new()),
            parser: Arc::new(Parser::new()),
            cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn load_from_directory(&mut self, providers_dir: &Path) -> Result<()> {
        let database = self.parser.parse_directory(providers_dir)?;
        self.database = Arc::new(database);
        Ok(())
    }

    pub fn get_database(&self) -> &Database {
        &self.database
    }

    pub fn get_providers_json(&self) -> Result<Value> {
        let cache_key = "providers".to_string();
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let json_value = serde_json::to_value(&*self.database)
            .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;

        self.cache.insert(cache_key, json_value.clone());
        Ok(json_value)
    }

    pub fn get_provider_json(&self, provider_id: &str) -> Result<Value> {
        let cache_key = format!("provider:{}", provider_id);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let provider = self.database.get_provider(provider_id)
            .ok_or_else(|| crate::error::Error::ProviderNotFound(provider_id.to_string()))?;

        let json_value = serde_json::to_value(provider)
            .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;

        self.cache.insert(cache_key, json_value.clone());
        Ok(json_value)
    }

    pub fn get_model_json(&self, provider_id: &str, model_id: &str) -> Result<Value> {
        let cache_key = format!("model:{}:{}", provider_id, model_id);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let model = self.database.get_model(provider_id, model_id)
            .ok_or_else(|| crate::error::Error::ModelNotFound(format!("{}/{}", provider_id, model_id)))?;

        let json_value = serde_json::to_value(model)
            .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;

        self.cache.insert(cache_key, json_value.clone());
        Ok(json_value)
    }

    pub fn get_model_schema(&self) -> Result<RootSchema> {
        let mut settings = SchemaSettings::openapi3();
        settings.inline_subschemas = true;
        
        let gen = SchemaGenerator::new(settings);
        let schema = gen.into_root_schema_for::<crate::models::Model>();
        
        Ok(schema)
    }

    pub fn get_all_model_ids(&self) -> Vec<String> {
        self.database
            .all_models()
            .map(|(provider_id, model_id, _)| format!("{}/{}", provider_id, model_id))
            .collect()
    }

    pub fn search_models(&self, query: &str) -> Vec<&crate::models::Model> {
        let query_lower = query.to_lowercase();
        
        self.database
            .all_models()
            .filter(|(_, _, model)| {
                model.name.to_lowercase().contains(&query_lower) ||
                model.id.to_lowercase().contains(&query_lower) ||
                model.family.as_ref().map_or(false, |f| f.to_lowercase().contains(&query_lower))
            })
            .map(|(_, _, model)| model)
            .collect()
    }

    pub fn get_models_by_family(&self, family: &str) -> Vec<&crate::models::Model> {
        self.database
            .all_models()
            .filter(|(_, _, model)| {
                model.family.as_ref().map_or(false, |f| f == family)
            })
            .map(|(_, _, model)| model)
            .collect()
    }

    pub fn get_models_by_capability(&self, capability: &str) -> Vec<&crate::models::Model> {
        self.database
            .all_models()
            .filter(|(_, _, model)| {
                match capability {
                    "reasoning" => model.reasoning,
                    "tool_call" => model.tool_call,
                    "attachment" => model.attachment,
                    "structured_output" => model.structured_output.unwrap_or(false),
                    "temperature" => model.temperature.unwrap_or(false),
                    _ => false,
                }
            })
            .map(|(_, _, model)| model)
            .collect()
    }

    pub fn get_models_by_modality(&self, modality: &str) -> Vec<&crate::models::Model> {
        let target_modality = match modality {
            "text" => crate::models::Modality::Text,
            "image" => crate::models::Modality::Image,
            "audio" => crate::models::Modality::Audio,
            "video" => crate::models::Modality::Video,
            "pdf" => crate::models::Modality::Pdf,
            _ => return Vec::new(),
        };

        self.database
            .all_models()
            .filter(|(_, _, model)| {
                model.modalities.input.contains(&target_modality) ||
                model.modalities.output.contains(&target_modality)
            })
            .map(|(_, _, model)| model)
            .collect()
    }

    pub fn get_statistics(&self) -> Value {
        let mut stats = serde_json::Map::new();
        
        stats.insert("providers".to_string(), Value::Number(self.database.provider_count().into()));
        stats.insert("models".to_string(), Value::Number(self.database.model_count().into()));
        
        let mut family_counts = std::collections::HashMap::new();
        let mut capability_counts = std::collections::HashMap::new();
        let mut modality_counts = std::collections::HashMap::new();
        
        for (_, _, model) in self.database.all_models() {
            // Count families
            if let Some(ref family) = model.family {
                *family_counts.entry(family.clone()).or_insert(0) += 1;
            }
            
            // Count capabilities
            if model.reasoning {
                *capability_counts.entry("reasoning".to_string()).or_insert(0) += 1;
            }
            if model.tool_call {
                *capability_counts.entry("tool_call".to_string()).or_insert(0) += 1;
            }
            if model.attachment {
                *capability_counts.entry("attachment".to_string()).or_insert(0) += 1;
            }
            if model.structured_output.unwrap_or(false) {
                *capability_counts.entry("structured_output".to_string()).or_insert(0) += 1;
            }
            
            // Count modalities
            for modality in &model.modalities.input {
                let modality_str = format!("{:?}", modality).to_lowercase();
                *modality_counts.entry(modality_str).or_insert(0) += 1;
            }
            for modality in &model.modalities.output {
                let modality_str = format!("{:?}", modality).to_lowercase();
                *modality_counts.entry(modality_str).or_insert(0) += 1;
            }
        }
        
        stats.insert("families".to_string(), Value::Object(
            family_counts.into_iter()
                .map(|(k, v)| (k, Value::Number(v.into())))
                .collect()
        ));
        
        stats.insert("capabilities".to_string(), Value::Object(
            capability_counts.into_iter()
                .map(|(k, v)| (k, Value::Number(v.into())))
                .collect()
        ));
        
        stats.insert("modalities".to_string(), Value::Object(
            modality_counts.into_iter()
                .map(|(k, v)| (k, Value::Number(v.into())))
                .collect()
        ));
        
        Value::Object(stats)
    }

    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
