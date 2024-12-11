use async_trait::async_trait;
use derive_new::new;
use kernel::repository::health::HealthCheckRepository;

use crate::database::ConnectionPool;

// 1. コンストラクタを自動生成させる
// コンストラクタとは
#[derive(new)]
pub struct HealthCheckRepositoryImpl {
    // 2. 構造体に`ConnectionPool`を持たせる
    db: ConnectionPool,
}

#[async_trait]
// 3. `HealthCheckRepository`を実装する
impl HealthCheckRepository for HealthCheckRepositoryImpl {
    async fn check_db(&self) -> bool {
        // 4. クエリ実行結果は`Result`型であるため
        // `OK` -> `true`
        //  `Err` -> `false`
        sqlx::query("SELECT 1")
            .fetch_one(self.db.inner_ref())
            .await
            .is_ok()
    }
}
