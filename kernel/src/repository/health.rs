use async_trait::async_trait;

#[mockall::automock]
#[async_trait] // 1
pub trait HealthCheckRepository: Send + Sync {
    // traitは"Send + Sync"を満たす必要がある。両者はどちらもマーカーとレイトである
    async fn check_db(&self) -> bool;
    // DBに接続し、接続を確立できるかを確認するための関数
    // 接続できた -> true
    // 接続できなかった -> false
}
