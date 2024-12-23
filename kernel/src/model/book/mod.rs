pub mod event;

// Bookのの内容を定義する
use crate::model::{
    id::{BookId, CheckoutId},
    user::{BookOwner, CheckoutUser},
};

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Book {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub owner: BookOwner,
    pub checkout: Option<Checkout>,
}

// ページネーションの範囲を指定するための設定値を格納する型を追加　
#[derive(Debug)]
pub struct BookListOptions{
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug)]
pub struct Checkout{
    pub checkout_id: CheckoutId,
    pub checked_out_by: CheckoutUser,
    pub checked_out_at: DateTime<Utc>,
}