use crate::error::{Result, VaultError};
use keyring::Entry;
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Metadata stored alongside keys
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyMetadata {
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub custom: HashMap<String, String>,
}

/// Trait for cross-platform key storage
pub trait KeyStorage: Send + Sync {
    fn store_key(&self, key_id: &str, key: &[u8], metadata: KeyMetadata) -> Result<()>;
    fn retrieve_key(&self, key_id: &str) -> Result<(Vec<u8>, KeyMetadata)>;
    fn delete_key(&self, key_id: &str) -> Result<()>;
    fn key_exists(&self, key_id: &str) -> bool;
    fn list_keys(&self) -> Result<Vec<String>>;
}

/// Windows implementation using DPAPI via keyring crate
#[cfg(target_os = "windows")]
pub struct WindowsKeyStorage {
    service_name: String,
}

#[cfg(target_os = "windows")]
impl WindowsKeyStorage {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }
    
    fn get_entry(&self, key_id: &str) -> Result<Entry> {
        Entry::new(&self.service_name, key_id)
            .map_err(|e| VaultError::Keychain(format!("Failed to create entry: {}", e)))
    }
    
    fn get_metadata_entry(&self, key_id: &str) -> Result<Entry> {
        let metadata_key = format!("{}_metadata", key_id);
        Entry::new(&self.service_name, &metadata_key)
            .map_err(|e| VaultError::Keychain(format!("Failed to create metadata entry: {}", e)))
    }
}

#[cfg(target_os = "windows")]
impl KeyStorage for WindowsKeyStorage {
    fn store_key(&self, key_id: &str, key: &[u8], metadata: KeyMetadata) -> Result<()> {
        // Store the key
        let entry = self.get_entry(key_id)?;
        let key_str = general_purpose::STANDARD.encode(key);
        entry
            .set_password(&key_str)
            .map_err(|e| VaultError::Keychain(format!("Failed to store key: {}", e)))?;
        
        // Store metadata separately
        let metadata_entry = self.get_metadata_entry(key_id)?;
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| VaultError::Keychain(format!("Failed to serialize metadata: {}", e)))?;
        metadata_entry
            .set_password(&metadata_json)
            .map_err(|e| VaultError::Keychain(format!("Failed to store metadata: {}", e)))?;
        
        Ok(())
    }
    
    fn retrieve_key(&self, key_id: &str) -> Result<(Vec<u8>, KeyMetadata)> {
        // Retrieve the key
        let entry = self.get_entry(key_id)?;
        let key_str = entry
            .get_password()
            .map_err(|e| VaultError::Keychain(format!("Failed to retrieve key: {}", e)))?;
        
        let key_data = general_purpose::STANDARD.decode(&key_str)
            .map_err(|e| VaultError::Keychain(format!("Failed to decode key: {}", e)))?;
        
        // Retrieve metadata
        let metadata_entry = self.get_metadata_entry(key_id)?;
        let metadata_json = metadata_entry
            .get_password()
            .map_err(|e| VaultError::Keychain(format!("Failed to retrieve metadata: {}", e)))?;
        
        let metadata: KeyMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| VaultError::Keychain(format!("Failed to parse metadata: {}", e)))?;
        
        Ok((key_data, metadata))
    }
    
    fn delete_key(&self, key_id: &str) -> Result<()> {
        // Delete key
        let entry = self.get_entry(key_id)?;
        entry
            .delete_password()
            .map_err(|e| VaultError::Keychain(format!("Failed to delete key: {}", e)))?;
        
        // Delete metadata
        let metadata_entry = self.get_metadata_entry(key_id)?;
        let _ = metadata_entry.delete_password(); // Ignore error if metadata doesn't exist
        
        Ok(())
    }
    
    fn key_exists(&self, key_id: &str) -> bool {
        self.get_entry(key_id)
            .and_then(|entry| {
                entry
                    .get_password()
                    .map(|_| true)
                    .map_err(|_| VaultError::Keychain("Key not found".to_string()))
            })
            .unwrap_or(false)
    }
    
    fn list_keys(&self) -> Result<Vec<String>> {
        // Note: keyring crate doesn't support listing all keys
        // This is a limitation of the OS keychain APIs
        // For now, return error indicating this limitation
        Err(VaultError::Keychain(
            "list_keys not supported by Windows Credential Manager API".to_string()
        ))
    }
}

/// macOS implementation (placeholder for future)
#[cfg(target_os = "macos")]
pub struct MacOSKeyStorage;

/// Linux implementation (placeholder for future)
#[cfg(target_os = "linux")]
pub struct LinuxKeyStorage;

/// Factory function to create platform-specific key storage
pub fn create_key_storage() -> Box<dyn KeyStorage> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsKeyStorage::new("identra-vault"))
    }
    
    #[cfg(target_os = "macos")]
    {
        // TODO: Implement macOS Keychain
        unimplemented!("macOS keychain not yet implemented")
    }
    
    #[cfg(target_os = "linux")]
    {
        // TODO: Implement Linux Secret Service
        unimplemented!("Linux Secret Service not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_keychain() {
        let storage = WindowsKeyStorage::new("identra-test");
        let test_key = b"test_secret_key_12345678901234567890";
        
        // Store key
        storage.store_key("test-key", test_key).unwrap();
        
        // Retrieve key
        let retrieved = storage.retrieve_key("test-key").unwrap();
        assert_eq!(test_key, &retrieved[..]);
        
        // Delete key
        storage.delete_key("test-key").unwrap();
        assert!(!storage.key_exists("test-key"));
    }
}
