use async_trait::async_trait;
use shared::error::AppResult;

use crate::model::{
    auth::{event::CreateToken, AccessToken},
    id::UserId,
};

#[async_trait]
pub trait AuthRepository: Send + Sync {
    // アクセストークンからユーザーIDを取得する
    async fn fetch_user_id_from_token(
        &self,
        access_token: &AccessToken,
    ) -> AppResult<Option<UserId>>;
    // メアドとパスが正しいかの検証　
    async fn verify_user (
        &self,
        email: &str,
        password: &str,
    ) -> AppResult<UserId>;
    // アクセストークンを生成する　
    async fn create_token(&self, event: CreateToken) -> AppResult<AccessToken>;
    // アクセストークンを削除する　
    async fn delete_token(&self, access_token: AccessToken) -> AppResult<()>;
}

