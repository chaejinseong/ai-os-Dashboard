pub mod gmail;
use crate::{model::Item, oauth::GoogleAuth};
use std::sync::Arc;
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("{0}")]
    Message(String),
}
impl From<String> for ConnectorError {
    fn from(s: String) -> Self {
        Self::Message(s)
    }
}
#[async_trait::async_trait]
pub trait Connector: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    async fn is_connected(&self) -> bool;
    async fn fetch(&self) -> Result<Vec<Item>, ConnectorError>;
}
pub type Registry = Vec<Arc<dyn Connector>>;
pub fn registry(auth: Arc<GoogleAuth>) -> Registry {
    vec![Arc::new(gmail::Gmail::new(auth))]
}
