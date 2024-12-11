use crate::model::book::{BookResponse, CreateBookRequest};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use kernel::model::id::BookId;
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

// 蔵書を登録するAPIを作成
pub async fn register_book(
    State(registry): State<AppRegistry>, // Appregistryを参照
    Json(req): Json<CreateBookRequest>,  // JSONデータから変換する構造体を指定する
) -> Result<StatusCode, AppError> {
    registry
        .book_repository()
        .create(req.into())
        .await
        .map(|_| StatusCode::CREATED)
}

// リクエストが正しく受け取れた場合
// メソッドがコールされる：引数にreqとしてリクエストデータにアクセスできるようになる
// メソッド内の処理
// State<AppRegistry>型のデータとしてAppRegistryの参照が引数で渡される -> トレイトメソッド越しにadapterのメソッドcreateを呼び出す(32行目)。

// 蔵書の一覧を取得するAPIを作成
pub async fn show_book_list(
    State(registry): State<AppRegistry>,
) -> Result<Json<Vec<BookResponse>>, AppError> {
    registry
        .book_repository()
        .find_all()
        .await
        .map(|v| v.into_iter().map(BookResponse::from).collect::<Vec<_>>())
        .map(Json) // 成功時の戻り値をJSON型で包んでいる:"Result<Vec<BookResponse>" -> "Result<Json<Vec<BookResponse>>"
        .map_err(AppError::from)
}

// idから蔵書を取得するAPI
pub async fn show_book(
    Path(book_id): Path<BookId>, // パスパラメーター取得のため:URLのパス構成(/books/uuid)となっていて、uuidの部分をuuidとして取得することができる
    State(registry): State<AppRegistry>,
) -> Result<Json<BookResponse>, AppError> {
    registry
        .book_repository()
        .find_by_id(book_id)
        .await
        .and_then(|bc| match bc {
            Some(bc) => Ok(Json(bc.into())),
            None => Err(AppError::EntityNotFound("not found".into())),
        })
        .map_err(AppError::from)
}
