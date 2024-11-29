use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::get, Router};
use tokio::net::TcpListener;

use sqlx::{postgres::PgConnectOptions, PgPool};

// どんなリクエストが来てもhelloworldを返す関数を作成　
// https://docs.rs/axum/0.7.5/axum/response/trait.IntoResponse.html
pub async fn health_check() -> StatusCode{
    StatusCode::OK
}

#[tokio::test]
async fn health_check_works(){

    // health関数を呼び出す。awaitとして結果を得る
    let status_code = health_check().await; 

    assert_eq!(status_code, StatusCode::OK);
}

// DBの接続設定を表す構造体を定義する
struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String, 
    pub database: String,
}

// DB接続設定から、Postgres接続用の構造体へ変更する
impl From<DatabaseConfig> for PgConnectOptions {
    fn from(cfg: DatabaseConfig) -> Self{
        Self::new()
            .host(&cfg.host)
            .port(cfg.port)
            .username(&cfg.username)
            .password(&cfg.password)
            .database(&cfg.database)
    }
}

// Postgres専用のコネクションプールを作成する　
fn connect_database_with(cfg: DatabaseConfig) -> PgPool{
    PgPool::connect_lazy_with(cfg.into())
}

async fn health_check_db(State(db): State<PgPool>) -> StatusCode{
    let connection_result = sqlx::query("SELECT 1").fetch_one(&db).await;
    match connection_result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[sqlx::test]
async fn health_check_db_works(pool: sqlx::PgPool) {
    let status_code = health_check_db(State(pool)).await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::main]
async fn main() -> Result<()> {

    // DB接続設定を定義する　
    let database_cfg = DatabaseConfig {
        host: "localhost".into(),
        port: 5433,
        username: "app".into(),
        password: "passwd".into(),
        database: "app".into(),
    };

    // コネクションプールを作る　
    let conn_pool = connect_database_with(database_cfg);

    // /helloというパスにGETリクエストが送られてきたらhello_word関数を呼び出す
    let app = Router::new()
        .route("/health", get(health_check))
        // ルーターの`State`にプールを登録しておき、各ハンドラで使えるようにする
        .route("/health/db", get(health_check_db))
        .with_state(conn_pool);
    
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);

    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);

    // サーバーを起動する　
    Ok(axum::serve(listener, app).await?)
}