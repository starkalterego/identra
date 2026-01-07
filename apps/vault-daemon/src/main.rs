use anyhow::Result;
use vault_daemon::{KeyStorage, VaultServer};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 Identra Vault Daemon starting...");
    println!("📍 Local secure storage initialized");
    
    // Initialize OS keychain
    let keychain = vault_daemon::keychain::create_key_storage();
    println!("🔑 OS Keychain integration active");
    
    // Initialize IPC server
    let server = VaultServer::new();
    server.start().await?;
    
    println!("✅ Vault Daemon ready");
    println!("🎯 Listening for IPC commands from Tauri...");
    
    // Keep daemon running
    tokio::signal::ctrl_c().await?;
    println!("🛑 Shutting down Vault Daemon...");
    
    Ok(())
}
