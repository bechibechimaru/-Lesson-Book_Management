use axum::{routing::post, Router};
use registry::AppRegistry;
use tracing::info;

use crate::handler::auth::{login, logout};

pub fn routes() -> Router<AppRegistry> {

    info!("Initializing routes for /auth/login and /auth/logout");

    let auth_router = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout));
    info!("Routes for _auth initialized");
    
    Router::new().nest("/auth", auth_router)
}
