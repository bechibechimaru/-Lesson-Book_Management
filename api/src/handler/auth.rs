// loginメソッドの実装　
use axum::{extract::State, http::StatusCode, Json};
use kernel::model::auth::event::CreateToken;
use registry::AppRegistry;
use shared::error::AppResult;

use crate::{
    extractor::AuthorizedUser,
    model::auth::{AccessTokenResponse, LoginRequest},
};

use tracing::{info, error};

pub async fn login(
    State(registry): State<AppRegistry>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AccessTokenResponse>> {
    let user_id = registry
        .auth_repository()
        .verify_user(&req.email, &req.password)
        .await?;
    let access_token = registry
        .auth_repository()
        .create_token(CreateToken::new(user_id))
        .await?;
    Ok(Json(AccessTokenResponse {
        user_id, 
        access_token: access_token.0,
    }))
}

pub async fn logout (
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode>{
    info!("Attempting to log out user ID: {:?}", user.id());
    match registry
        .auth_repository()
        .delete_token(user.access_token)
        .await {
            Ok(_) => {
                // 成功時のログ　
                Ok(StatusCode::NO_CONTENT)
            }
            Err(e) => {
                // エラー時の詳細なログ　
                error!(
                    "Error occured while attempting to delete token for user ID",                    
                );
                Err(e)
            }
        }
    
}