pub mod model;

use redis::{AsyncCommands, Client};
use shared::{config::RedisConfig, error::AppResult};

use self::model::{RedisKey, RedisValue};

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    // newメソッド：Redis接続用のクライアントを初期化する
    pub fn new(config: &RedisConfig) -> AppResult<Self> {  
        let client = 
            Client::open(format!("redis://{}:{}", config.host, config.port))?;
            Ok(Self { client })
    }
    // 16行目: `Client::open`でRedisサーバーに接続するクライアントを初期化する・戻り値：Result<Client, redis::RedisError>
    // 16行目: `format!("redis://{}:{}", config.host, config.port)`でRedisサーバーのURIを生成する
    // 16行目: `?`でエラーが発生した場合、処理を終了し、呼び出し元に返す
    // 17行目：`Ok`が帰ってきた場合(成功した場合)、RedisClientを生成して返す

    // set_exメソッド：期限付きでキーとバリューを保存する。
    pub async fn set_ex<T: RedisKey>(
        &self,
        key: &T,
        value: &T::Value,
        ttl: u64,
    ) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.set_ex(key.inner(), value.inner(), ttl).await?;
        Ok(())
    }

    // 30行目：set_exメソッドで、Redisのキーに値を設定する
    // key.innner(): RedisKeyトレイとのメソッドで、Redisに設定するキーの文字列を取得する　
    // value.inner(): 値を取得するメソッド。Redisに保存するデータを表す。
    // ttl：キーの有効期限を秒単位で指定する。

    // キーを指定してバリューを取り出す
    pub async fn get<T: RedisKey>(
        &self,
        key: &T,
    ) -> AppResult<Option<T::Value>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: Option<String> = conn.get(key.inner()).await?;
        result.map(T::Value::try_from).transpose()
    }

    // キーを指定して、Redis上の該当のキーとバリューを削除する
    pub async fn delete<T: RedisKey>(&self, key: &T) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.del(key.inner()).await?;
        Ok(())
    }

    // 接続確認：ヘルスチェック
    pub async fn try_connect(&self) -> AppResult<()> {
        let _ = self.client.get_multiplexed_async_connection().await?;
        Ok(())
    }
}
