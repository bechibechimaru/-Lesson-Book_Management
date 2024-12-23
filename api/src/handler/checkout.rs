//　ユーザーリクエストを処理するエンドポイントを作成する
use crate::{extractor::AuthorizedUser, model::checkout::CheckoutsResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use kernel::model::{
    checkout::event::{CreateCheckout, UpdateReturned},
    id::{BookId, CheckoutId},
};
use registry::AppRegistry;
use shared::error::AppResult;
use tracing::info;

pub async fn checkout_book(
    user: AuthorizedUser,
    Path(book_id): Path<BookId>,// HTTPのパスパラメーターから`book_id`を取得している
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let create_checkout_history = 
        CreateCheckout::new(book_id, user.id(), chrono::Utc::now());

    let result = registry
        .checkout_repository()
        .create(create_checkout_history)
        .await
        .map(|_| StatusCode::OK);

        info!("The endpoint of checkout_book request successfully worked.");

        result
}

pub async fn return_book(
    user: AuthorizedUser,
    Path((book_id, checkout_id,)): Path<(BookId, CheckoutId)>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let update_returned = UpdateReturned::new(
        checkout_id,
        book_id,
        user.id(),
        chrono::Utc::now(),
    );

    let result = registry
        .checkout_repository()
        .update_returned(update_returned)
        .await
        .map(|_| StatusCode::OK);
    info!("The endpoint of return_book request successfully worked.");
    result
}

pub async fn show_checked_out_list(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    let result = registry
        .checkout_repository()
        .find_unreturned_all()
        .await
        .map(CheckoutsResponse::from)
        .map(Json);

    info!("The endpoint of show_checked_out_list request successfully worked.");

    result
}

pub async fn checkout_history(
    _user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    let result = registry
        .checkout_repository()
        .find_history_by_book_id(book_id)
        .await
        .map(CheckoutsResponse::from)
        .map(Json);
    
    info!("The endpoint of checkout_history request successfully worked.");

    result
}