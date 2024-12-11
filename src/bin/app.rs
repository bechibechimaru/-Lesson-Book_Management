use adapter::database::connect_database_with;
use anyhow::Context;
use anyhow::{Error, Result};
use axum::Router;
use registry::AppRegistry;
use shared::config::AppConfig;
use shared::env::{which, Environment};
use std::net::{Ipv4Addr, SocketAddr};
use std::result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use api::route::{book::build_book_routers, health::build_health_check_routers};

use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;

use tower_http::cors::{self, CorsLayer};

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

// サーバー起動分のログを生成する
async fn bootstrap() -> Result<()> {
    // `AppConfig`を生成させる
    let app_config = AppConfig::new()?;

    // DBへの接続を行う、コネクションプールを取り出す
    let pool = connect_database_with(&app_config.database);

    // `AppRegistry`を生成する
    let registry = AppRegistry::new(pool);

    // `build_health_check_routers`関数をcall. `AppRegistry`を`Router`に登録。
    let app = Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .layer(cors())
        // 以下にリクエストとレスポンス時にログを出力するレイヤーを追加する
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .with_state(registry);

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
