use kernel::model::{id::UserId, role::Role, user::User};
use shared::error::AppError;
use sqlx::types::chrono::{DateTime, Utc};
use std::str::FromStr;
use tracing::{info, error};

pub struct UserRow{
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub role_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;
    fn try_from(value: UserRow) -> Result<Self, Self::Error> {
        // 変換開始のログ　
        info!("Starting conversion from USerRow to USer. UserRow");
        let UserRow{
            user_id,
            name,
            email,
            role_name,
            ..
        } = value;

        let role = match Role::from_str(role_name.as_str()) {
            Ok(role) => {
                // 成功時のログ
                info!("Successfully converted role_name: {:?} to Role: {:?}", role_name, role);
                role
            }
            Err(e) => {
                // エラー時のログ
                error!(
                    "Failed to convert role_name: {:?} to Role. Error: {:?}",
                    role_name, e
                );
                return Err(AppError::ConversionEntityError(e.to_string()));
            }
        };

        info!("Successfully converted UserRow to User");

        Ok(User {
            id: user_id, 
            name,
            email,
            role,
        })
        
    }
}