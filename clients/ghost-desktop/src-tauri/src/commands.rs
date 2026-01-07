use tauri::{State, AppHandle, Manager};
use crate::state::{NexusState, VaultStatus};
use crate::ipc_client::VaultClient;
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct SystemStatusResponse {
    pub vault_status: VaultStatus,
    pub active_identity: Option<String>,
    pub enclave_connection: bool,
    pub security_level: String,
}

#[tauri::command]
pub async fn get_system_status(state: State<'_, NexusState>) -> Result<SystemStatusResponse, String> {
    let status = state.status.lock().map_err(|_| "State mutex poisoned")?.clone();
    let identity = state.active_identity.lock().map_err(|_| "Identity mutex poisoned")?.clone();
    
    // TODO: Hook up actual AWS Nitro Enclave ping
    let enclave_alive = true; 

    Ok(SystemStatusResponse {
        vault_status: status,
        active_identity: identity,
        enclave_connection: enclave_alive,
        security_level: "MAXIMUM".to_string(),
    })
}

#[tauri::command]
pub async fn vault_memory(
    state: State<'_, NexusState>, 
    content: String
) -> Result<String, String> {
    if content.trim().is_empty() {
        return Err("Payload empty.".to_string());
    }

    // FIXME: Replace with actual identra-crypto impl
    let _encrypted_blob = format!("ENC::{}", content); 
    
    {
        let mut metrics = state.metrics.lock().map_err(|_| "Metrics mutex poisoned")?;
        metrics.memory_encrypted += content.len();
    }

    let id = Uuid::new_v4();
    println!("[NEXUS] Vaulted memory block: {}", id);

    Ok(format!("Secured block [{}]", id.to_string().split('-').next().unwrap()))
}

#[tauri::command]
pub async fn toggle_launcher(app: AppHandle) -> Result<(), String> {
    let launcher = app.get_webview_window("launcher").ok_or("ERR_NO_WINDOW")?;
    
    if launcher.is_visible().unwrap_or(false) {
        launcher.hide().map_err(|e| e.to_string())?;
    } else {
        launcher.show().map_err(|e| e.to_string())?;
        launcher.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_main_window(app: AppHandle) -> Result<(), String> {
    let main = app.get_webview_window("main").ok_or("ERR_NO_WINDOW")?;
    
    if main.is_visible().unwrap_or(false) {
        main.hide().map_err(|e| e.to_string())?;
    } else {
        main.show().map_err(|e| e.to_string())?;
        main.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ================ VAULT IPC COMMANDS ================

#[tauri::command]
pub async fn vault_store_key(identity_id: String, key: Vec<u8>) -> Result<String, String> {
    let mut client = VaultClient::connect()
        .await
        .map_err(|e| format!("Failed to connect to vault: {}", e))?;
    
    client.store_key(identity_id, key)
        .await
        .map_err(|e| format!("Failed to store key: {}", e))
}

#[tauri::command]
pub async fn vault_retrieve_key(identity_id: String) -> Result<Vec<u8>, String> {
    let mut client = VaultClient::connect()
        .await
        .map_err(|e| format!("Failed to connect to vault: {}", e))?;
    
    client.retrieve_key(identity_id)
        .await
        .map_err(|e| format!("Failed to retrieve key: {}", e))
}

#[tauri::command]
pub async fn vault_delete_key(identity_id: String) -> Result<String, String> {
    let mut client = VaultClient::connect()
        .await
        .map_err(|e| format!("Failed to connect to vault: {}", e))?;
    
    client.delete_key(identity_id)
        .await
        .map_err(|e| format!("Failed to delete key: {}", e))
}

#[tauri::command]
pub async fn vault_key_exists(identity_id: String) -> Result<bool, String> {
    let mut client = VaultClient::connect()
        .await
        .map_err(|e| format!("Failed to connect to vault: {}", e))?;
    
    client.key_exists(identity_id)
        .await
        .map_err(|e| format!("Failed to check key existence: {}", e))
}