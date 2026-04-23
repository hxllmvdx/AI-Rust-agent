use crate::services::session_store::SessionStore;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub sessions: SessionStore,
}
