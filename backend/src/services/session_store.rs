use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    error::BackendError,
    models::sessions::{ConversationMessage, SessionState},
};

#[derive(Clone)]
pub struct SessionStore {
    client: redis::Client,
    ttl: u64,
}

impl SessionStore {
    pub fn new(client: redis::Client, ttl: u64) -> Self {
        Self { client, ttl }
    }

    fn session_key(session_id: Uuid) -> String {
        format!("session:{session_id}")
    }

    pub async fn create_session(&self) -> Result<Uuid, BackendError> {
        let session_id = Uuid::new_v4();
        let session = SessionState {
            session_id,
            messages: Vec::new(),
        };

        let key = Self::session_key(session_id);
        let value = serde_json::to_string(&session)?;

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(key, value, self.ttl).await?;

        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: Uuid) -> Result<SessionState, BackendError> {
        let key = Self::session_key(session_id);

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(json) => {
                let session: SessionState = serde_json::from_str(&json)?;
                Ok(session)
            }
            None => Err(BackendError::SessionNotFound),
        }
    }

    pub async fn reset_session(&self, session_id: Uuid) -> Result<(), BackendError> {
        let mut session = self.get_session(session_id).await?;
        session.messages.clear();

        let key = Self::session_key(session_id);
        let value = serde_json::to_string(&session)?;

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(key, value, self.ttl).await?;

        Ok(())
    }

    pub async fn update_session(
        &self,
        session_id: Uuid,
        messages: ConversationMessage,
    ) -> Result<(), BackendError> {
        let mut session = self.get_session(session_id).await?;
        session.messages.push(messages);

        let key = Self::session_key(session_id);
        let value = serde_json::to_string(&session)?;

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(key, value, self.ttl).await?;

        Ok(())
    }
}
