// kernelレイヤーで定義されているCheckoutをクライアントにJSONで返すための構造の定義を行う

use chrono::{DateTime, Utc};
use kernel::model::{
    checkout::{Checkout, CheckoutBook},
    id::{BookId, CheckoutId, UserId},
};
use serde::Serialize;


#[derive(Serialize)] // 構造体をシリアライズ可能にする：JSON形式への変換に対応
#[serde(rename_all = "camelCase")] // フィールド名がJSONエンコード時にcamelCaseになる
pub struct CheckoutsResponse {
    pub items: Vec<CheckoutResponse>,
}

impl From<Vec<Checkout>> for CheckoutsResponse {
    // 型変換：Vec<Checkout> into CheckoutsResponse
    fn from(value: Vec<Checkout>) -> Self{
        Self{
            items: value
                    .into_iter() // バリューをイテレーターに変換する、各要素を順に処理する準備
                    .map(CheckoutResponse::from)// from関数を呼び出し
                    .collect(),// 変換された要素をVec<Checkoutresponse>に収集する
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutResponse {
    pub id: CheckoutId,
    pub checked_out_by: UserId,
    pub checked_out_at: DateTime<Utc>,
    pub returned_at: Option<DateTime<Utc>>,
    pub book: CheckoutBookResponse,
}

impl From<Checkout> for CheckoutResponse {
    fn from(value: Checkout) -> Self {
        // 構造体の分解：Checkout構造体方各フィールドを取り出す
        let Checkout {
            id,
            checked_out_by,
            checked_out_at,
            returned_at,
            book,
        } = value;
        Self {
            id,
            checked_out_by,
            checked_out_at,
            returned_at,
            book: book.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutBookResponse {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
}

impl From<CheckoutBook> for CheckoutBookResponse {
    fn from(value: CheckoutBook) -> Self {

        // レスポンスであるCheckoutBookをSelf(CheckoutBookResponse)に型変換を行う
        let CheckoutBook {
            book_id,
            title,
            author,
            isbn,
        } = value;

        Self {
            id: book_id, // 変数名：値
            title,
            author,
            isbn,
        }
    }
}