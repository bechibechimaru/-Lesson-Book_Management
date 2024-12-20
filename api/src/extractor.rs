use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::{async_trait, RequestPartsExt};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use kernel::model::auth::AccessToken;
use kernel::model::id::UserId;
use kernel::model::role::Role;
use kernel::model::user::User;
use shared::error::AppError;

use registry::AppRegistry;
use tracing::{info, error};

// a) リクエストの前処理を実行後、handlerに渡す構造体を定義　
pub struct AuthorizedUser{
    pub access_token: AccessToken,
    pub user: User,
}

impl AuthorizedUser {
    pub fn id(&self) -> UserId {
        self.user.id
    }

    pub fn is_admin(&self) -> bool{
        self.user.role == Role::Admin
    }
}

#[async_trait]
impl FromRequestParts<AppRegistry> for AuthorizedUser {
    type Rejection = AppError;

    // handlerメソッドの引数にAuthoraizedUserを追加したときはこのメソッドが呼ばれる
    async fn from_request_parts(
        parts: &mut Parts,
        registry: &AppRegistry,
    ) -> Result<Self, Self::Rejection> {
        // b) HTTPヘッダからアクセストークンを取り出す　
        let TypedHeader(Authorization(bearer)) = parts// 取り出したtokenが正しいかを型(TypedHeader)で判断
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::UnauthorizedError)?;
        let access_token = AccessToken(bearer.token().to_string());
        info!("Successfully Extracted AccessToken from Header: 1/3");

        // アクセストークンが紐づくユーザーIDを抽出する　
        let user_id = registry
            .auth_repository()
            .fetch_user_id_from_token(&access_token)
            .await?
            .ok_or(AppError::UnauthenticatedError)?;
        info!("Successfully extracted UserId linked AccessToken: 2/3");

        // ユーザーIDでDBからユーザーのレコードを引く　

        let user = match registry.user_repository().find_current_user(user_id).await {
            Ok(Some(user)) => {
                info!("Successfully found user record for UserID: {:?}", user_id);
                user
            }
            Ok(None) => {
                info!("No user record found for UserID: {:?}", user_id);
                return Err(AppError::UnauthenticatedError);
            }
            Err(e) => {
                error!(
                    "Error occurred while fetching user record for UserID: {:?}. Error: {:?}",
                    user_id, e
                );
                return Err(e);
            }
        };

info!("Successfully completed user lookup process for UserID: {:?} 3/3", user_id);
Ok(Self { access_token, user })

        
    }
}
