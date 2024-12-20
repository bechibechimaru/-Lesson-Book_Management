use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use garde::Validate;
use kernel::model::{book::event::DeleteBook, id::BookId};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

use crate::{
    extractor::AuthorizedUser,
    model::book::{
        BookListQuery, BookResponse, CreateBookRequest, PaginatedBookResponse,
        UpdateBookRequest, UpdateBookRequestWithIds
    },
};

// 蔵書を登録するAPIを作成
pub async fn register_book(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>, // Appregistryを参照
    Json(req): Json<CreateBookRequest>,  // JSONデータから変換する構造体を指定する
) -> AppResult<StatusCode> {
    req.validate(&())?;

    registry
        .book_repository()
        .create(req.into(), user.id())
        .await
        .map(|_| StatusCode::CREATED)
}

// リクエストが正しく受け取れた場合
// メソッドがコールされる：引数にreqとしてリクエストデータにアクセスできるようになる
// メソッド内の処理
// State<AppRegistry>型のデータとしてAppRegistryの参照が引数で渡される -> トレイトメソッド越しにadapterのメソッドcreateを呼び出す(32行目)。

// 蔵書の一覧を取得するAPIを作成
pub async fn show_book_list(
    _user: AuthorizedUser,
    Query(query): Query<BookListQuery>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<PaginatedBookResponse>>{
    query.validate(&())?;

    registry
        .book_repository()
        .find_all(query.into())
        .await
        .map(PaginatedBookResponse::from)
        .map(Json) // 成功時の戻り値をJSON型で包んでいる:"Result<Vec<BookResponse>" -> "Result<Json<Vec<BookResponse>>"
}

// idから蔵書を取得するAPI
pub async fn show_book(
    _user: AuthorizedUser,
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

pub async fn update_book(
    user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateBookRequest>,
) -> AppResult<StatusCode> {
    req.validate(&())?;

    let update_book = UpdateBookRequestWithIds::new(book_id, user.id(), req);

    registry   
        .book_repository()
        .update(update_book.into())
        .await
        .map(|_| StatusCode::OK)
}

pub async fn delete_book(
    user: AuthorizedUser, 
    Path(book_id): Path<BookId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let delete_book = DeleteBook {
        book_id,
        requested_user: user.id(),
    };
    registry 
        .book_repository()
        .delete(delete_book)
        .await
        .map(|_| StatusCode::OK)
}