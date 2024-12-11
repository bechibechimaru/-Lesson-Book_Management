// 認証に使うデータ型の定義
use shared::error::{AppError, AppResult};
use std::str::FromStr;

use kernel::model::{
    auth::{event::CreateToken, AccessToken},
    id::UserId,
};

use crate::redis::model::{RedisKey, RedisValue};

pub struct UserItem{
    pub user_id: UserId,
    pub password_hash: String,
}

pub struct AuthorizationKey(String);
pub struct AuthorizedUserId(UserId);

// token内容をコピーする
pub fn from(event:CreateToken) -> (AuthorizationKey, AuthorizedUserId){
    (
        AuthorizationKey(event.access_token),
        AuthorizedUserId(event.user_id),
    )
}

// From: 
impl From<AuthorizationKey> for AccessToken {
    fn from(key: AuthorizationKey) -> Self {
        // 構造体（タプル）を指定する：要素が1つでも0と定義しなければならない
        Self(key.0)
    }
}

impl From<AccessToken> for AuthorizationKey {
    fn from(token: AccessToken) -> Self{
        Self(token.0)
    }
}

impl From<&AccessToken> for AuthorizationKey {
    fn from(token: &AccessToken) -> Self{
        Self(token.0.to_string())
    }
}

impl RedisKey for AuthorizationKey {
    type Value = AuthorizedUserId;

    fn inner(&self) -> String {
        self.0.clone()
    }
}

impl RedisValue for AuthorizedUserId {
    fn inner(&self) -> String{
        self.0.to_string()
    }
}

impl TryFrom<String> for AuthorizedUserId {
    type Error = AppError;

    fn try_from(s: String) -> AppResult<Self> {
        Ok(Self(UserId::from_str(&s).map_err(|e|{
            AppError::ConversionEntityError(e.to_string())
        })?))
    }
}

impl AuthorizedUserId{
    pub fn into_inner(self) -> UserId {
        self.0
    }
}