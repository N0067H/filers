use async_trait::async_trait;

#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn put(&self, key: &str, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>;
    async fn get(&self, key: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error>>;
}
