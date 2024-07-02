use async_trait::async_trait;
use uuid::Uuid;
#[async_trait]
// FIXME: Pending system design surrounding llm-determined retrievals
// Maybe we just resrict retrievals for now along some predefined params?
pub trait Repository<T> {
    async fn save(&self, new: T) -> anyhow::Result<T>;
    async fn retrieve_by_id(&self, id: Uuid) -> anyhow::Result<T>;
    async fn delete_by_id(&self, id: Uuid) -> anyhow::Result<T>;
}
