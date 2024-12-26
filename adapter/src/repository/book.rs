use async_trait::async_trait;
use derive_new::new;
use tracing::info;

use kernel::model::{
    id::{BookId, UserId},
    book::{event::DeleteBook, Checkout}, 
    list::PaginatedList,
};
use kernel::{
    model::book::{
        event::{CreateBook, UpdateBook},
        Book, BookListOptions,
    },
    repository::book::BookRepository,
};
use shared::error::{AppError, AppResult};

use std::collections::HashMap;

use crate::database::model::book::{BookRow, BookCheckoutRow,PaginatedBookRow};
use crate::database::ConnectionPool;

#[derive(new)]
pub struct BookRepositoryImpl {
    // DB接続を保持する
    db: ConnectionPool,
}

#[async_trait]
impl BookRepository for BookRepositoryImpl {
    async fn create(
        &self, 
        event: CreateBook, 
        user_id: UserId,
    ) -> AppResult<()> {
        sqlx::query!(
            // SQLクエリ
            r#"
                INSERT INTO books (title, author, isbn, description, user_id)
                VALUES($1, $2, $3, $4, $5)
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            user_id as _
        )
        .execute(self.db.inner_ref()) // "execute"：SQLクエリをDBに送信し、結果を実行するメソッド
        .await // "await"との違いは、errをどう返すか
        // sqlx::Error型をAppError型に変換
        .map_err(AppError::SpecificOperationError)?;

        info!("Book created successfully: title='{}', author='{}', isbn='{}', user_id={}", 
        event.title, 
        event.author, 
        event.isbn, 
        user_id);

        Ok(())
    }
    
    // A,B,C,Dという処理が存在する。Aの処理に異常な時間がかかるとする。
    // その時、Aをfeatureに入れ処理する。また、B,C,Dに関してはAと並行して処理を進める。

    // ページネーション
    // 1. 指定したlimitとoffsetの範囲に該当する蔵書IDのリストと総件数を取得する
    // 2. 対象の蔵書IDから蔵書のレコードデータを取得する
    // 3. 取得したデータの戻り値の型に合うよう整えて返す

    // 総件数を取得する時、1つ目のクエリのレコードカラムに総件数が含まれるような実装であるため、このクエリ結果のレコードが0件の時は総件数も取得できない。
    // その時は、総件数も0件として返す仕様としている 

    async fn find_all(
        &self,
        options: BookListOptions,
    ) -> AppResult<PaginatedList<Book>> {
        let BookListOptions { limit, offset } = options;

        let rows: Vec<PaginatedBookRow> = sqlx::query_as!(
            PaginatedBookRow,
            r#"
                SELECT 
                    COUNT(*) OVER() AS "total!",
                    b.book_id AS id                
                FROM books as b
                ORDER BY b.created_at DESC
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        // レコードが1つもない時はtotalも0にする
        let total = rows.first().map(|r| r.total).unwrap_or_default();
        let book_ids = rows.into_iter().map(|r| r.id).collect::<Vec<BookId>>();

        info!("The number of total books were successfully counted: find_all(1/2)");

        let rows: Vec<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT 
                    b.book_id AS book_id,
                    b.title AS title,
                    b.author AS author,
                    b.isbn AS isbn,
                    b.description AS description,
                    u.user_id AS owned_by,
                    u.name AS owner_name
                FROM books AS b
                INNER JOIN users AS u using(user_id)
                WHERE b.book_id IN (SELECT * FROM UNNEST($1::uuid[]))
                ORDER BY b.created_at DESC
            "#,
            &book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let book_ids = 
            rows.iter().map(|book| book.book_id).collect::<Vec<_>>();

        let mut checkouts = self.find_checkouts(&book_ids).await?;
        let items = rows
            .into_iter()
            .map(|row|{
                let checkout = checkouts.remove(&row.book_id);
                row.into_book(checkout)
            })
            .collect();

        info!("Books was successfully selected: find_all(2/2)");

        Ok(PaginatedList {
            total,
            limit,
            offset,
            items,
        })
    }

    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>> {
        let row: Option<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT 
                    b.book_id AS book_id,
                    b.title AS title,
                    b.author AS author,
                    b.isbn AS isbn,
                    b.description AS description,
                    u.user_id AS owned_by,
                    u.name AS owner_name
                FROM books AS b
                INNER JOIN users AS u USING(user_id)
                WHERE book_id = $1
            "#,
            book_id as _ // query_as!マクロによるコンパイル時の型チェックを無効化
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        info!("The book successfully selected.: find_by_id (1/1)");

        match row {
            Some(r) => {
                let checkout = self
                    .find_checkouts(&[r.book_id])
                    .await?
                    .remove(&r.book_id);
                Ok(Some(r.into_book(checkout)))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, event: UpdateBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                UPDATE books
                SET
                    title = $1,
                    author = $2,
                    isbn = $3,
                    description = $4
                WHERE book_id = $5
                AND user_id = $6
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            event.book_id as _,
            event.requested_user as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1{
            return Err(AppError::EntityNotFound(
                "specified book not found".into(),  
            ));
        }

        // 成功時のログ内容
        info!(
            "Book updated successfully: book_id={}, title='{}', user_id={}",
            event.book_id,
            event.title,
            event.requested_user
        );
        
        Ok(())
    }

    async fn delete(&self, event: DeleteBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                DELETE FROM books
                WHERE book_id = $1
                AND   user_id = $2
            "#,
            event.book_id as _,
            event.requested_user as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound(
                "specified book not found".into(),
            ));
        }

        info!("The book successfully deleted: book_id = {}", event.book_id);

        Ok(())
        
    }
}

impl BookRepositoryImpl{
    // 指定されたbook_idが貸出中の場合に貸出情報を返すメソッドを追加する
    async fn find_checkouts(
        &self,
        book_ids: &[BookId],
    ) -> AppResult<HashMap<BookId, Checkout>> {
        let res = sqlx::query_as!(
            BookCheckoutRow,
            r#"
                SELECT 
                    c.checkout_id,
                    c.book_id,
                    u.user_id,
                    u.name AS user_name,
                    c.checked_out_at
                FROM checkouts AS c
                INNER JOIN users AS u using(user_id)
                WHERE book_id = ANY($1)
                ;
            "#,
            book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(|checkout| (checkout.book_id, Checkout::from(checkout)))
        .collect();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::book::BookRepositoryImpl;
    use kernel::model::id::UserId;
    use std::str::FromStr;

    #[sqlx::test(fixtures("common", "book"))]
    async fn test_update_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        // 2. fixtures/book.sqlで作成済みの書籍を取得　
        let book_id = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b").unwrap();
        let book = repo.find_by_id(book_id).await?.unwrap();
        const NEW_AUTHOR: &str = "更新後の著者名";
        assert_ne!(book.author, NEW_AUTHOR);

        // 3. 書籍の更新用のパラメーターを作成し、更新を行う
        let update_book = UpdateBook {
            book_id: book.id,
            title: book.title,
            author: NEW_AUTHOR.into(),
            isbn: book.isbn,
            description: book.description,
            requested_user: UserId::from_str("5b4c96ac-316a-4bee-8e69-cac5eb84ff4c").unwrap(),
        };
        repo.update(update_book).await.unwrap();

        // 4. 更新後の書籍を取得し、期待通りに更新されていることを検証する　
        let book = repo.find_by_id(book_id).await?.unwrap();
        assert_eq!(book.author, NEW_AUTHOR);

        Ok(())
    }
}
