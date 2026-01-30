use identra_proto::vault::{
    vault_service_server::{VaultService, VaultServiceServer},
    StoreKeyRequest, StoreKeyResponse,
    RetrieveKeyRequest, RetrieveKeyResponse,
    DeleteKeyRequest, DeleteKeyResponse,
    ListKeysRequest, ListKeysResponse,
    KeyExistsRequest, KeyExistsResponse,
};
use crate::ipc_client::VaultClient;
use tonic::{Request, Response, Status};

pub struct VaultServiceImpl;

impl VaultServiceImpl {
    pub fn new() -> Self {
        Self
    }
    
    pub fn into_server(self) -> VaultServiceServer<Self> {
        VaultServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl VaultService for VaultServiceImpl {
    async fn store_key(
        &self,
        request: Request<StoreKeyRequest>,
    ) -> Result<Response<StoreKeyResponse>, Status> {
        let req = request.into_inner();
        
        let mut client = VaultClient::connect()
            .await
            .map_err(|e| Status::unavailable(format!("Vault daemon not available: {}", e)))?;
        
        // Convert protobuf expires_at (Timestamp) to Unix timestamp
        let expires_at = req.expires_at.map(|ts| ts.seconds);
        
        client.store_key(
            req.key_id.clone(), 
            req.key_data,
            req.metadata,
            expires_at,
        )
            .await
            .map_err(|e| Status::internal(format!("Failed to store key: {}", e)))?;
        
        tracing::info!("Stored key: {}", req.key_id);
        
        Ok(Response::new(StoreKeyResponse {
            success: true,
            message: format!("Key '{}' stored successfully", req.key_id),
        }))
    }
    
    async fn retrieve_key(
        &self,
        request: Request<RetrieveKeyRequest>,
    ) -> Result<Response<RetrieveKeyResponse>, Status> {
        let req = request.into_inner();
        
        let mut client = VaultClient::connect()
            .await
            .map_err(|e| Status::unavailable(format!("Vault daemon not available: {}", e)))?;
        
        let (key_data, metadata, created_at, expires_at) = client.retrieve_key(req.key_id.clone())
            .await
            .map_err(|e| Status::not_found(format!("Key not found: {}", e)))?;
        
        tracing::info!("Retrieved key: {}", req.key_id);
        
        // Convert Unix timestamp to protobuf Timestamp
        let created_at_ts = Some(prost_types::Timestamp {
            seconds: created_at,
            nanos: 0,
        });
        
        Ok(Response::new(RetrieveKeyResponse {
            key_data,
            metadata,
            created_at: created_at_ts,
        }))
    }
    
    async fn delete_key(
        &self,
        request: Request<DeleteKeyRequest>,
    ) -> Result<Response<DeleteKeyResponse>, Status> {
        let req = request.into_inner();
        
        let mut client = VaultClient::connect()
            .await
            .map_err(|e| Status::unavailable(format!("Vault daemon not available: {}", e)))?;
        
        client.delete_key(req.key_id.clone())
            .await
            .map_err(|e| Status::internal(format!("Failed to delete key: {}", e)))?;
        
        tracing::info!("Deleted key: {}", req.key_id);
        
        Ok(Response::new(DeleteKeyResponse {
            success: true,
            message: format!("Key '{}' deleted successfully", req.key_id),
        }))
    }
    
    async fn list_keys(
        &self,
        _request: Request<ListKeysRequest>,
    ) -> Result<Response<ListKeysResponse>, Status> {
        let mut client = VaultClient::connect()
            .await
            .map_err(|e| Status::unavailable(format!("Vault daemon not available: {}", e)))?;
        
        let key_ids = client.list_keys()
            .await
            .map_err(|e| {
                tracing::warn!("list_keys not supported: {}", e);
                // Windows Credential Manager doesn't support listing
                Status::unimplemented("list_keys not supported by OS keychain")
            })?;
        
        tracing::info!("Listed {} keys", key_ids.len());
        
        Ok(Response::new(ListKeysResponse {
            key_ids,
            next_page_token: String::new(),
        }))
    }
    
    async fn key_exists(
        &self,
        request: Request<KeyExistsRequest>,
    ) -> Result<Response<KeyExistsResponse>, Status> {
        let req = request.into_inner();
        
        let mut client = VaultClient::connect()
            .await
            .map_err(|e| Status::unavailable(format!("Vault daemon not available: {}", e)))?;
        
        let exists = client.key_exists(req.key_id.clone())
            .await
            .map_err(|e| Status::internal(format!("Failed to check key existence: {}", e)))?;
        
        Ok(Response::new(KeyExistsResponse { exists }))
    }
}

impl Default for VaultServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}
