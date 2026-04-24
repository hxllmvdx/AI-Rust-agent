use redis::{AsyncCommands, RedisResult};
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

    fn session_meta_key(session_id: Uuid) -> String {
        format!("session:{session_id}:meta")
    }

    fn session_messages_key(session_id: Uuid) -> String {
        format!("session:{session_id}:messages")
    }

    pub async fn create_session(&self) -> Result<Uuid, BackendError> {
        let session_id = Uuid::new_v4();
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let meta_key = Self::session_meta_key(session_id);
        let messages_key = Self::session_messages_key(session_id);

        let _: () = redis::pipe()
            .atomic()
            .set_ex(&meta_key, "1", self.ttl)
            .del(&messages_key)
            .query_async(&mut conn)
            .await?;

        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: Uuid) -> Result<SessionState, BackendError> {
        let meta_key = Self::session_meta_key(session_id);
        let messages_key = Self::session_messages_key(session_id);

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let exists: bool = conn.exists(&meta_key).await?;
        if !exists {
            return Err(BackendError::SessionNotFound);
        }

        let values: Vec<String> = conn.lrange(messages_key, 0, -1).await?;
        let messages = values
            .into_iter()
            .map(|json| serde_json::from_str(&json))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SessionState {
            session_id,
            messages,
        })
    }

    pub async fn reset_session(&self, session_id: Uuid) -> Result<(), BackendError> {
        self.ensure_session_exists(session_id).await?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let meta_key = Self::session_meta_key(session_id);
        let messages_key = Self::session_messages_key(session_id);

        let _: () = redis::pipe()
            .atomic()
            .del(&messages_key)
            .expire(&meta_key, self.ttl as i64)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn append_message(
        &self,
        session_id: Uuid,
        message: ConversationMessage,
    ) -> Result<(), BackendError> {
        self.ensure_session_exists(session_id).await?;
        let value = serde_json::to_string(&message)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let meta_key = Self::session_meta_key(session_id);
        let messages_key = Self::session_messages_key(session_id);

        let _: () = redis::pipe()
            .atomic()
            .rpush(&messages_key, value)
            .expire(&messages_key, self.ttl as i64)
            .expire(&meta_key, self.ttl as i64)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    async fn ensure_session_exists(&self, session_id: Uuid) -> Result<(), BackendError> {
        let meta_key = Self::session_meta_key(session_id);
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let exists: RedisResult<bool> = conn.exists(meta_key).await;

        match exists {
            Ok(true) => Ok(()),
            Ok(false) => Err(BackendError::SessionNotFound),
            Err(err) => Err(BackendError::RedisError(err)),
        }
    }
}
