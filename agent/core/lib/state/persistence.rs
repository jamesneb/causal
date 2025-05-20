// agent/core/lib/state/persistence.rs

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tracing::{debug, error, info, warn};

// Default paths for state persistence
pub const DEFAULT_STATE_DIR: &str = "/tmp/lambda-extension-state";
pub const DEFAULT_STATE_FILE: &str = "state.json";
pub const DEFAULT_BACKUP_FILE: &str = "state.backup.json";

// Trait for serializable state
pub trait PersistentState: Serialize + for<'de> Deserialize<'de> + Default {
    // Get state ID (used for file naming)
    fn state_id() -> &'static str;
    
    // Custom validation logic
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

// State persistence manager
pub struct StatePersistence<T: PersistentState> {
    state_dir: String,
    state_file: String,
    backup_file: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T: PersistentState> StatePersistence<T> {
    // Create a new state persistence manager
    pub fn new(state_dir: Option<&str>, state_file: Option<&str>, backup_file: Option<&str>) -> Self {
        let state_dir = state_dir.unwrap_or(DEFAULT_STATE_DIR);
        let state_file = state_file.unwrap_or(DEFAULT_STATE_FILE);
        let backup_file = backup_file.unwrap_or(DEFAULT_BACKUP_FILE);
        
        Self {
            state_dir: state_dir.to_string(),
            state_file: state_file.to_string(),
            backup_file: backup_file.to_string(),
            _marker: std::marker::PhantomData,
        }
    }
    
    // Get full path to state file
    fn state_path(&self) -> String {
        format!("{}/{}", self.state_dir, self.state_file)
    }
    
    // Get full path to backup file
    fn backup_path(&self) -> String {
        format!("{}/{}", self.state_dir, self.backup_file)
    }
    
    // Ensure state directory exists
    fn ensure_dir(&self) -> Result<()> {
        if !Path::new(&self.state_dir).exists() {
            fs::create_dir_all(&self.state_dir)
                .context("Failed to create state directory")?;
        }
        Ok(())
    }
    
    // Load state from disk
    pub fn load(&self) -> Result<T> {
        self.ensure_dir()?;
        
        let state_path = self.state_path();
        let backup_path = self.backup_path();
        
        // Try loading from primary state file
        let result = self.load_from_file(&state_path);
        
        if result.is_err() {
            warn!("Failed to load state from primary file, trying backup");
            
            // Try loading from backup if primary fails
            let backup_result = self.load_from_file(&backup_path);
            
            if backup_result.is_err() {
                warn!("Failed to load state from backup, creating new default state");
                // Return default state if both files fail
                return Ok(T::default());
            }
            
            // Restore from backup
            let state = backup_result?;
            info!("Successfully restored state from backup");
            
            // Save the recovered state to the primary file
            self.save(&state)?;
            
            Ok(state)
        } else {
            Ok(result?)
        }
    }
    
    // Load state from a specific file
    fn load_from_file(&self, path: &str) -> Result<T> {
        if !Path::new(path).exists() {
            return Err(anyhow::anyhow!("State file does not exist"));
        }
        
        // Open and read file
        let mut file = File::open(path).context("Failed to open state file")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).context("Failed to read state file")?;
        
        // Deserialize
        let state: T = serde_json::from_str(&contents).context("Failed to deserialize state")?;
        
        // Validate
        state.validate()?;
        
        Ok(state)
    }
    
    // Save state to disk
    pub fn save(&self, state: &T) -> Result<()> {
        self.ensure_dir()?;
        
        let state_path = self.state_path();
        let backup_path = self.backup_path();
        
        // First, make a backup of the current state file if it exists
        if Path::new(&state_path).exists() {
            fs::copy(&state_path, &backup_path)
                .context("Failed to create backup before saving")?;
        }
        
        // Serialize state
        let contents = serde_json::to_string_pretty(state)
            .context("Failed to serialize state")?;
        
        // Write to temporary file first to avoid partial writes
        let temp_path = format!("{}.tmp", state_path);
        let mut temp_file = File::create(&temp_path)
            .context("Failed to create temporary state file")?;
        
        temp_file.write_all(contents.as_bytes())
            .context("Failed to write state data")?;
        temp_file.flush()
            .context("Failed to flush state data")?;
        
        // Rename temporary file to final location (atomic operation on most filesystems)
        fs::rename(&temp_path, &state_path)
            .context("Failed to finalize state file")?;
        
        debug!("State saved successfully");
        Ok(())
    }
    
    // Delete state files
    pub fn clear(&self) -> Result<()> {
        let state_path = self.state_path();
        let backup_path = self.backup_path();
        
        // Remove state file if it exists
        if Path::new(&state_path).exists() {
            fs::remove_file(&state_path)
                .context("Failed to remove state file")?;
        }
        
        // Remove backup file if it exists
        if Path::new(&backup_path).exists() {
            fs::remove_file(&backup_path)
                .context("Failed to remove backup file")?;
        }
        
        debug!("State cleared successfully");
        Ok(())
    }
}

// Utility functions for working with binary state
pub mod binary {
    use super::*;
    use std::io::{self, Cursor};
    
    // Save binary data with CRC32 checksum
    pub fn save_binary_with_crc<P: AsRef<Path>>(path: P, data: &[u8]) -> io::Result<()> {
        // Compute CRC32
        let crc = crate::common::utils::error_correction::compute_crc32(data);
        
        // Create file
        let mut file = File::create(path)?;
        
        // Write data length
        file.write_all(&(data.len() as u32).to_le_bytes())?;
        
        // Write CRC32
        file.write_all(&crc.to_le_bytes())?;
        
        // Write data
        file.write_all(data)?;
        
        Ok(())
    }
    
    // Load binary data and verify CRC32 checksum
    pub fn load_binary_with_crc<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        // Open file
        let mut file = File::open(path)?;
        
        // Read length
        let mut len_bytes = [0u8; 4];
        file.read_exact(&mut len_bytes)?;
        let data_len = u32::from_le_bytes(len_bytes) as usize;
        
        // Read CRC32
        let mut crc_bytes = [0u8; 4];
        file.read_exact(&mut crc_bytes)?;
        let expected_crc = u32::from_le_bytes(crc_bytes);
        
        // Read data
        let mut data = vec![0u8; data_len];
        file.read_exact(&mut data)?;
        
        // Verify CRC32
        let actual_crc = crate::common::utils::error_correction::compute_crc32(&data);
        if actual_crc != expected_crc {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "CRC32 checksum mismatch"
            ));
        }
        
        Ok(data)
    }
    
    // Compress and save binary data
    pub fn save_compressed_binary<P: AsRef<Path>>(path: P, data: &[u8]) -> io::Result<()> {
        // Compress data
        let compressed = crate::common::utils::compression::compress_data(
            data,
            flate2::Compression::default()
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Save with CRC
        save_binary_with_crc(path, &compressed)
    }
    
    // Load and decompress binary data
    pub fn load_compressed_binary<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        // Load with CRC check
        let compressed = load_binary_with_crc(path)?;
        
        // Decompress
        let decompressed = crate::common::utils::compression::decompress_data(&compressed)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        Ok(decompressed)
    }
}
