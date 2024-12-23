// DBとのやりとりを描く
// 貸出状態によって取得・書き込みの対象となるテーブルを分けて考える必要がある
// → トランザクションを用いて複数回のクエリ操作を行う

use crate::database::{
    model::checkout::{CheckoutRow, CheckoutStateRow, ReturnedCheckoutRow},
    ConnectionPool
};
use async_trait::async_trait;
use derive_new::new;
use kernel::model::checkout::{
    event::{CreateCheckout, UpdateReturned},
    Checkout,
};
use kernel::model::id::{BookId, CheckoutId, UserId};
use kernel::repository::checkout::CheckoutRepository;
use shared::error::{AppError, AppResult};

#[derive(new)]
pub struct CheckoutRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl CheckoutRepository for CheckoutRepositoryImpl {
    // 貸出操作を行う　
    async fn create(&self, event: CreateCheckout) -> AppResult<()>{
        let mut tx = self.db.begin().await?;

        // トランザクション分離レベルをSERIALIZABLEに設定する
        self.set_transaction_serializable(&mut tx).await?;

        // 事前のチェックとして以下を調べる　
        // - 指定の蔵書IDを持つ蔵書が存在するか　
        // - 存在した場合：この蔵書は貸出中ではないか

        // 上記がYESだった場合、このブロック以降の処理に進む
        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT 
                        b.book_id,
                        c.checkout_id AS "checkout_id?: CheckoutId",
                        NULL AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id as _
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                // 指定した書籍が存在しない場合　
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "書籍({})が見つかりませんでした。",
                        event.book_id
                    )))
                }
                // 指定した書籍が存在するが貸出中の場合
                Some(CheckoutStateRow{
                    checkout_id: Some(_),
                    .. 
                }) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        "書籍({})に対する貸出がすでに存在しています。",
                        event.book_id
                    )))
                } 
                _ => {} //それ以外は処理続行
            }
        }

        // 貸出処理を行う checkoutsテーブルにレコードを追加する
        let checkout_id = CheckoutId::new();
        let res = sqlx::query!(
            r#"
                INSERT INTO checkouts
                (checkout_id, book_id, user_id, checked_out_at)
                VALUES ($1, $2, $3, $4);
            "#,
            checkout_id as _,
            event.book_id as _,
            event.checked_out_by as _,
            event.checked_out_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been created" .into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    // 返却操作を行う
    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()>{
        let mut tx = self.db.begin().await?;

        // トランザクションの分離レベルをSERIALIZABLEに設定する
        self.set_transaction_serializable(&mut tx).await?;

        // 返却操作時のチェック項目
        // - 指定した蔵書IDを持つ蔵書が存在するのか
        // - 存在した場合
        //  - この蔵書は貸出中、かつ、借りたユーザーが指定のユーザーと同じか

        // 上記がYesの場合このブロック以降の処理に進む
        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT 
                        b.book_id,
                        c.checkout_id AS "checkout_id?: CheckoutId",
                        c.user_id AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id as _,
            )
            .fetch_optional(&mut tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                // 指定した書籍がない場合　
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "書籍({})が見つかりませんでした。",event.book_id
                    )))
                }

                // 指定した書籍が貸出中であり、貸出IDまたは借りたユーザーが異なる場合
                Some(CheckoutStateRow {
                    checkout_id: Some(c), 
                    user_id: Some(u),
                    .. 
                }) if(c, u) != (event.checkout_id, event.returned_by) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        "指定の貸出ID(({}), ユーザー({}), 書籍({}))は返却できません",
                        event.checkout_id,
                        event.returned_by,
                        event.book_id
                    )))
                }
                _ => {} // それ以外は処理続行
            }      
        }

        // DB上の返却操作として、checkoutsテーブルにアツ当該当貸出IDのレコードを、returned_atを追加して、returned_checkoutsテーブルにINSERTする。
        let res = sqlx::query!(
            r#"
                INSERT INTO returned_checkouts
                (checkout_id, book_id, user_id, checked_out_at, returned_at)
                SELECT checkout_id, book_id, user_id, checked_out_at, $2
                FROM checkouts 
                WHERE checkout_id = $1
                ;
            "#,
            event.checkout_id as _,
            event.returned_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::SpecificOperationError(
                "No returning record has been updated".into(),
            ));
        }

        // 上記処理が成功したら、checkoutsテーブルから該当貸出IDのレコードを削除する　
        let res = sqlx::query!(
            r#"
                DELETE FROM checkouts WHERE checkout_id = $1;
            "#,
            event.checkout_id as _,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1{
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been deleted" .into()
            ));
        } 

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    // 全ての未返却の貸出情報を取得する。
    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>> {
        // checkoutsテーブルにあるレコードを全件抽出する
        // booksテーブルとINNTER JOINし、蔵書の情報も一緒に抽出する
        // 出力するレコードは、貸出日の古い順に並べる
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT 
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                ORDER BY c.checked_out_at ASC
                ;
            "#,
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    // ユーザーIDに紐づく未返却の貸出情報を取得する。
    async fn find_unreturned_by_user_id(
        &self,
        user_id: UserId,
    ) -> AppResult<Vec<Checkout>> {
        // find_unreturned_allのSQLにユーザーIDで絞り込むWHERE句を追加したものである。
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT 
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.user_id = $1
                ORDER BY c.checked_out_at ASC
                ;
            "#,
            user_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    // 蔵書の貸出履歴(返却済みも含む)を取得する。
    async fn find_history_by_book_id(
        &self,
        book_id: BookId,
    ) -> AppResult<Vec<Checkout>> {
        // 未返却の貸出情報を取得
        let checkout: Option<Checkout> =
            self.find_unreturned_by_book_id(book_id).await?;

        // 返却済みの貸出情報を取得　
        let mut checkout_histories: Vec<Checkout> = sqlx::query_as!(
            ReturnedCheckoutRow,
            r#"
                SELECT 
                    rc.checkout_id,
                    rc.book_id,
                    rc.user_id,
                    rc.checked_out_at,
                    rc.returned_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM returned_checkouts AS rc
                INNER JOIN books AS b USING(book_id)
                WHERE rc.book_id = $1
                ORDER BY rc.checked_out_at DESC
            "#,
            book_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Checkout::from)
        .collect();

        // 貸出中である場合は返却済みの履歴の先頭に追加する
        if let Some(co) = checkout {
            checkout_histories.inner(0, co);
        }

        Ok(checkout_histories)
        
    }
}

impl CheckoutRepositoryImpl {
    // トランザクション分離レベルをSERIALIZABLEにするために内部的に使うメソッド

    async fn set_transaction_serializable (
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> AppResult<()> {
        sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    // find_history_by_book_idで内部的に使うメソッド
    async fn find_unreturned_by_book_id(
        &self,
        book_id: BookId
    ) -> AppResult<Option<Checkout>> {
        let res = sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT 
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.book_id = $1
            "#,
            book_id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .map(Checkout::from);

        Ok(res)
    }
}
