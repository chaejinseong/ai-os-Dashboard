pub mod github;
pub mod gmail;
pub mod google_calendar;
pub mod notion;
use crate::{model::Item, oauth::GoogleAuth, store::Store};
use std::sync::{Arc, Mutex};
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
pub fn registry(auth: Arc<GoogleAuth>, store: Arc<Mutex<Store>>) -> Registry {
    vec![
        Arc::new(gmail::Gmail::new(auth.clone())),
        Arc::new(google_calendar::GoogleCalendar::new(auth)),
        Arc::new(notion::Notion::new(store.clone())),
        Arc::new(github::GitHub::new(store)),
    ]
}
