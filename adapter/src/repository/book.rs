use async_trait::async_trait;
use derive_new::new;
use tracing::info;

use kernel::model::{
    id::{BookId, UserId},
    {book::event::DeleteBook, list::PaginatedList},
};
use kernel::{
    model::book::{
        event::{CreateBook, UpdateBook},
        Book, BookListOptions,
    },
    repository::book::BookRepository,
};
use shared::error::{AppError, AppResult};

use crate::database::model::book::{BookRow, PaginatedBookRow};
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

        let items = rows.into_iter().map(Book::from).collect();

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

        Ok(row.map(Book::from))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::user::UserRepositoryImpl;
    use kernel::{
        model::user::event::CreateUser, repository::user::UserRepository
    };

    #[sqlx::test]
    async fn test_register_book(pool: sqlx::PgPool) -> anyhow::Result<()> {

        sqlx::query!(r#"INSERT INTO roles(name) VALUES ('Admin'), ('User');"#)
            .execute(&pool)
            .await?;

        let user_repo = 
            UserRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        // テスト用のユーザーデータを作成
        let user = user_repo 
            .create(CreateUser {
                name: "Test User".into(),
                email: "test@example.com".into(),
                password: "test_password".into(),
            })
            .await?;

        // テスト用の蔵書データを作成
        let book = CreateBook {
            title: "Test Title".into(),
            author: "Test Author".into(),
            isbn: "Test ISBN".into(),
            description: "Test Description".into(),
        };
        // 蔵書データを投入すると正常に動作することを確認
        repo.create(book, user.id).await?;

        // find_allを実行するためにはBookListOptions型の値が必要
        let options = BookListOptions{
            limit: 20,
            offset: 0,
        };

        // 蔵書の一覧を取得すると投入した1件だけ取得できることを確認
        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);

        let book_id = res.items[0].id;
        let res = repo.find_by_id(book_id).await?;
        assert!(res.is_some());

        // 取得した蔵書データがCreateBookで投入した、蔵書データと一致することを確認
        let Book {
            id,
            title,
            author,
            isbn,
            description,
            owner,
        } = res.unwrap();
        assert_eq!(id, book_id);
        assert_eq!(title, "Test Title");
        assert_eq!(author, "Test Author");
        assert_eq!(isbn, "Test ISBN");
        assert_eq!(description, "Test Description");
        assert_eq!(owner.name, "Test User");

        Ok(())
    }
}
