use kernel::model::{
    book::{event::CreateBook, Book},
    id::BookId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookRequest {
    pub title: String,
    pub author: String,
    pub isbn: String,
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

// BookResponseの定義：データの取得の際の応答形式を作成

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

impl From<Book> for BookResponse {
    fn from(value: Book) -> Self {
        let Book {
            id,
            title,
            author,
            isbn,
            description,
        } = value;
        Self {
            id,
            title,
            author,
            isbn,
            description,
        }
    }
}

// 上記まででリクエストに必要な方は作成完了
// 処理は/handler/book.rsで実装

// 2行目のserdeに関して
// "serde::Deserialize"はserdeが提供するトレイト：リクエストに含まれるJSONをRustのデータに変換する（Deserialize）
// Convert Data
// Rust -> something: Serialize
// something -> Rust: Deserialize

// 6行目のserdeに関して
// 構造体全体のフィールド名を所定のルールでリネームするために使うアトリビュートである
