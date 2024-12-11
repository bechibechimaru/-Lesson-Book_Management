use std::env;
use strum::EnumString;

#[derive(Default, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Environment {
    // 開発環境向けで動作していることを示す
    #[default]
    Development,
    // 本番環境向けで動作していることを示す
    Production,
}

/// 開発環境・本番環境のどちら向けのビルドであるかを示す
pub fn which() -> Environment {
    // debug_assertionsがon：デバッグビルド
    // else: リリースビルド
    // 以下のlet defalut_new = ~　は片方のみが実行される

    #[cfg(debug_assertions)]
    let default_env = Environment::Development;
    #[cfg(not(debug_assertions))]
    let default_env = Environment::Production;

    match env::var("ENV") {
        Err(_) => default_env,
        Ok(v) => v.parse().unwrap_or(default_env),
    }
}

// 環境変数ENVにおいて、productionと指定されていれば本番環境向け
// 環境変数ENVにおいて、developmentと指定されていれば開発環境向け
// 上記に当てはまらない場合
// 本アプリケーションのビルドがリリースビルド：本番環境向け
// 本アプリケーションのビルドがデバッグビルド：開発環境向け
