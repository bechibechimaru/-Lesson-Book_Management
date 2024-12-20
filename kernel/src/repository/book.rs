use async_trait::async_trait;
use shared::error::AppResult;

use crate::model::{
    book::{
        event::{CreateBook,DeleteBook, UpdateBook},
        Book, BookListOptions,
    },
    id::{BookId, UserId}, // BookId型をuseする
    list::PaginatedList,
};

#[async_trait]
pub trait BookRepository: Send + Sync {
    // 蔵書を追加する際に所有者を指定するuser_idを引数に追加
    async fn create(
        &self, 
        event: CreateBook,
        user_id: UserId,
    ) -> AppResult<()>;
    // ページネーションするためにoptions引数を追加し,戻り値はVecからPaginatedList型に変更
    async fn find_all(
        &self,
        options: BookListOptions,
    ) -> AppResult<PaginatedList<Book>>;
    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>>;
    async fn update(&self, event: UpdateBook) -> AppResult<()>;
    async fn delete(&self, event: DeleteBook) -> AppResult<()>;
}