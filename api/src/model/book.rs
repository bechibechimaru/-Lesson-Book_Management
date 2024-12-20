use derive_new::new;
use garde::Validate;
use kernel::model::{
    book::{
        event::{CreateBook, UpdateBook},
        Book, BookListOptions,
    },
    id::{BookId, UserId},
    list::PaginatedList,
};
use serde::{Deserialize, Serialize};

use super::user::BookOwner;


#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookRequest {
    #[garde(length(min=1))]
    pub title: String,
    #[garde(length(min=1))]
    pub author: String,
    #[garde(length(min=1))]
    pub isbn: String,
    #[garde(skip)]
    pub description: String,
}

// CreateBookへのFromトレイト実装：データの変換を行う
impl From<CreateBookRequest> for CreateBook {
    fn from(value: CreateBookRequest) -> Self {
        let CreateBookRequest {
            title,
            author,
            isbn,
            description,
        } = value;
        Self {
            title,
            author,
            isbn,
            description,
        }
    }
}

// 蔵書データの更新用の型を追加する
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookRequest {
    #[garde(length(min=1))]
    pub title: String,
    #[garde(length(min=1))]
    pub author: String,
    #[garde(length(min=1))]
    pub isbn: String,
    #[garde(skip)]
    pub description: String,
}

#[derive(new)]
pub struct UpdateBookRequestWithIds(BookId, UserId, UpdateBookRequest);
impl From<UpdateBookRequestWithIds> for UpdateBook {
    fn from(value: UpdateBookRequestWithIds) -> Self {
        let UpdateBookRequestWithIds(
            book_id,
            user_id,
            UpdateBookRequest {
                title,
                author,
                isbn,
                description,
            },
        ) = value;

        UpdateBook {
            book_id,
            title,
            author,
            isbn,
            description,
            requested_user: user_id,
        }
    }
}

// クエリでlimitとoffsetを受け取るための型
// handler側のメソッドで、クエリのデータを取得できる　
#[derive(Debug, Deserialize, Validate)]
pub struct BookListQuery{
    #[garde(range(min=0))]
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[garde(range(min=0))]
    #[serde(default)]
    pub offset: i64,
}

const DEFAULT_LIMIT: i64 = 20;
const fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

impl From<BookListQuery> for BookListOptions {
    fn from(value: BookListQuery) -> Self{
        let BookListQuery { limit, offset} = value;
        Self { limit, offset }
    }
}

// BookResponseの定義：データの取得の際の応答形式を作成
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub owner: BookOwner
}

impl From<Book> for BookResponse {
    fn from(value: Book) -> Self {
        let Book {
            id,
            title,
            author,
            isbn,
            description,
            owner,
        } = value;
        
        Self {
            id,
            title,
            author,
            isbn,
            description,
            owner: owner.into(),
        }
    }
}

// 上記まででリクエストに必要な型は作成完了
// 処理は/handler/book.rsで実装

// 2行目のserdeに関して
// "serde::Deserialize"はserdeが提供するトレイト：リクエストに含まれるJSONをRustのデータに変換する（Deserialize）
// Convert Data
// Rust -> something: Serialize
// something -> Rust: Deserialize

// 6行目のserdeに関して
// 構造体全体のフィールド名を所定のルールでリネームするために使うアトリビュートである

// apiレイヤーでのページネーション表現用の型
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedBookResponse{
    pub total: i64, 
    pub limit: i64,
    pub offset: i64, 
    pub items: Vec<BookResponse>,
}

impl From<PaginatedList<Book>> for PaginatedBookResponse {
    fn from(value: PaginatedList<Book>) -> Self {
        let PaginatedList {
            total,
            limit,
            offset,
            items,
        } = value;

        Self{
            total,
            limit,
            offset,
            items: items.into_iter().map(BookResponse::from).collect(),
        }
    }
}