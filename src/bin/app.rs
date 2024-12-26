use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use adapter::{database::connect_database_with, redis::RedisClient};
use anyhow::{Context, Result};
use axum::{http::Method, Router};
use registry::AppRegistryImpl;
use shared::config::AppConfig;
use shared::env::{which, Environment};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use api::route::{auth, v1};

use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tower_http::cors::{self, CorsLayer};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    bootstrap().await
}

// ロガーを初期化する関数
fn init_logger() -> Result<()> {
    let log_level = match which() {
        Environment::Development => "debug",
        Environment::Production => "info",
    };

    // ログレベルを設定
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());

    // ログレベル出力形式を設定
    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        .try_init()?;

    Ok(())
}

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_origin(cors::Any)
}

// サーバー起動分のログを生成する
async fn bootstrap() -> Result<()> {
    // `AppConfig`を生成させる
    let app_config = AppConfig::new()?;

    // DBへの接続を行う、コネクションプールを取り出す
    let pool = connect_database_with(&app_config.database);
    
    let kv = Arc::new(RedisClient::new(&app_config.redis)?);

    // `AppRegistry`を生成する
    let registry = Arc::new(AppRegistryImpl::new(pool, kv, app_config));

    // `build_health_check_routers`関数をcall. `AppRegistry`を`Router`に登録。
    let app = Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        // 以下にリクエストとレスポンス時にログを出力するレイヤーを追加する
        .layer(cors())
        .with_state(registry); // AppRegistry

    // 起動時と起動失敗時のログを設定する
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // prinln!からtracing::info!に変更
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app)
        .await
        .context("Unexpected error happened in server")
        // 起動失敗した際のエラーログをtracing::error!で出力
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Unexpected error"
            )
        })
}

