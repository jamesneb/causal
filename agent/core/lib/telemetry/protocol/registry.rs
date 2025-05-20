// agent/core/lib/telemetry/protocol/registry.rs

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

// Protocol constants
pub const PROTOCOL_FIELD_REGISTRY_VERSION: u32 = 1;

// Metric field registry - maps field names to IDs for efficient binary encoding
pub static FIELD_REGISTRY: Lazy<RwLock<FieldRegistry>> = Lazy::new(|| {
    RwLock::new(FieldRegistry::new())
});

// Field registry for mapping between field names and IDs
pub struct FieldRegistry {
    name_to_id: HashMap<String, u8>,
    id_to_name: HashMap<u8, String>,
    next_id: u8,
    reserved_ids: HashMap<String, u8>,
}

impl FieldRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            name_to_id: HashMap::new(),
            id_to_name: HashMap::new(),
            next_id: 10, // Start after reserved IDs
            reserved_ids: HashMap::new(),
        };
        
        // Reserve IDs for common fields
        registry.reserve_id("timestamp", 1);
        registry.reserve_id("request_id", 2);
        registry.reserve_id("memory_usage_mb", 3);
        registry.reserve_id("cpu_usage_percent", 4);
        registry.reserve_id("duration_ms", 5);
        registry.reserve_id("error", 6);
        registry.reserve_id("function_name", 7);
        registry.reserve_id("function_version", 8);
        registry.reserve_id("region", 9);
        
        registry
    }
    
    pub fn reserve_id(&mut self, name: &str, id: u8) {
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.insert(id, name.to_string());
        self.reserved_ids.insert(name.to_string(), id);
    }
    
    pub fn get_id_for_field(&self, name: &str) -> u8 {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }
        
        // We're in a read-only context and need to assign a new ID
        // This should not happen in normal usage with proper initialization
        // Return a default value or log an error
        tracing::warn!("Field '{}' not found in registry", name);
        0 // Return 0 (Null) as a fallback
    }
    
    pub fn register_and_get_id_for_field(&mut self, name: &str) -> u8 {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }
        
        // Assign new ID
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id < 10 {
            // Wrapped around, try to find unused IDs
            for i in 10..=255 {
                if !self.id_to_name.contains_key(&i) {
                    self.next_id = i;
                    break;
                }
            }
        }
        
        // Register the new mapping
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.insert(id, name.to_string());
        
        id
    }
    
    pub fn get_field_for_id(&self, id: u8) -> Option<&str> {
        self.id_to_name.get(&id).map(|s| s.as_str())
    }
    
    // Serialize registry for transmission
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(self.name_to_id.len() * 10);
        
        // Write version
        buffer.extend_from_slice(&PROTOCOL_FIELD_REGISTRY_VERSION.to_le_bytes());
        
        // Write count
        buffer.extend_from_slice(&(self.name_to_id.len() as u16).to_le_bytes());
        
        // Write mappings
        for (name, &id) in &self.name_to_id {
            buffer.push(id);
            buffer.push(name.len() as u8);
            buffer.extend_from_slice(name.as_bytes());
        }
        
        buffer
    }
    
    // Deserialize registry from received data
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 6 {
            return Err(anyhow!("Invalid field registry data"));
        }
        
        let mut registry = Self::new();
        
        // Read version
        let mut version_bytes = [0u8; 4];
        version_bytes.copy_from_slice(&data[0..4]);
        let version = u32::from_le_bytes(version_bytes);
        
        if version != PROTOCOL_FIELD_REGISTRY_VERSION {
            return Err(anyhow!("Unsupported field registry version"));
        }
        
        // Read count
        let mut count_bytes = [0u8; 2];
        count_bytes.copy_from_slice(&data[4..6]);
        let count = u16::from_le_bytes(count_bytes) as usize;
        
        // Read mappings
        let mut offset = 6;
        for _ in 0..count {
            if offset + 2 > data.len() {
                return Err(anyhow!("Invalid field registry data"));
            }
            
            let id = data[offset];
            let name_len = data[offset + 1] as usize;
            offset += 2;
            
            if offset + name_len > data.len() {
                return Err(anyhow!("Invalid field registry data"));
            }
            
            let name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
            offset += name_len;
            
            // Don't overwrite reserved IDs
            if !registry.reserved_ids.contains_key(&name) {
                registry.name_to_id.insert(name.clone(), id);
                registry.id_to_name.insert(id, name);
            }
        }
        
        // Update next_id to avoid conflicts
        let max_id = registry.name_to_id.values().copied().max().unwrap_or(9);
        registry.next_id = max_id.wrapping_add(1);
        if registry.next_id < 10 {
            registry.next_id = 10;
        }
        
        Ok(registry)
    }
    
    // Create a schema representation
    pub fn create_schema(&self) -> HashMap<String, String> {
        let mut schema = HashMap::new();
        for (name, _) in &self.name_to_id {
            // In a real implementation, you might want to include type information here
            schema.insert(name.clone(), "any".to_string());
        }
        schema
    }
}

// Schema type for defining expected fields and their types
pub struct Schema {
    pub fields: HashMap<String, String>,
    pub required_fields: Vec<String>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            required_fields: Vec::new(),
        }
    }

    pub fn add_field(&mut self, name: String, field_type: String, required: bool) {
        self.fields.insert(name.clone(), field_type);
        if required {
            self.required_fields.push(name);
        }
    }

    pub fn validate(&self, data: &serde_json::Value) -> Result<()> {
        if !data.is_object() {
            return Err(anyhow!("Data is not an object"));
        }

        let obj = data.as_object().unwrap();

        // Check required fields
        for field in &self.required_fields {
            if !obj.contains_key(field) {
                return Err(anyhow!("Missing required field: {}", field));
            }
        }

        // Check field types (simplified)
        for (name, expected_type) in &self.fields {
            if let Some(value) = obj.get(name) {
                let actual_type = match value {
                    serde_json::Value::Null => "null",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::Object(_) => "object",
                };
                
                if expected_type != "any" && expected_type != actual_type {
                    return Err(anyhow!("Field '{}' has wrong type. Expected '{}', got '{}'", 
                        name, expected_type, actual_type));
                }
            }
        }

        Ok(())
    }
}
