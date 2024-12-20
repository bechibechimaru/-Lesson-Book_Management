# それぞれのファイルの役割

## adapter:外部処理を担う

* DB処理
  * 接続
  * SQL実行
* 外部のAPI処理

## api：リクエストの受付処理

## kernel：ドメインロジック・ビジネスロジック

## registry

## shared

FASTPOS

infrastructure
presentation
application
domain
shared

---

# 具体的な実装

## 5.2.6　registryモジュールの実装

### 完了したこと

* DB接続
* レコードの投入

### このセクションで行うこと

* apiを通じて上記のメソッドを呼び出せるようにする

### 具体的な実装例

#### registryに、BookRepositoryImplを参照する実装を行う

1. kernel/BookRepositoryトレイト/findallメソッドを呼び出す
2. BookrepositoryImplのfind_allメソッドを呼び出す

## 5.2.7 apiレイヤーの実装

### 完了したこと

* リポジトリの実装
* モジュールレジストリへの実装

### このセクションで行うこと

* ハンドラの実装
  * ハンドラで行う処理

```text
・HTTPリクエストが受信した後に、中身を解析、必要な情報を取り出す　
・その情報をもとに、例えばリポジトリメソッドを呼び出してDBにアクセスするなどして、必要なデータを取り出す
・最後にHTTPレスポンスとして返せる形にデータを加工して返す
```

* ルーターの実装
  * パスを定義する。routeディレクトリは以下のファイルに実装する

### 具体的な実装例

1. 依存クレートの追加
2. ファイルおよびディレクトリの追加
3. ハンドラとルーターの定義
4. モデルの定義
5. リクエスト受信時の実装
6. 動作確認

## 5.4　ユーザー管理機能の実装

### ログイン機能シーケンス図

```mermaid
sequenceDiagram
  Client ->> API: Login
  API ->> DB: Get UserInfo
  DB ->> API: Response 
  API ->> API: Compare UserInfo
  API ->> API: Generate Access Token
  API ->> Redis: Store Access Token
  Redis ->> API: StatusCode
  API ->> Client: Return Access Token

```

### Redis接続のシーケンス図

```mermaid
sequenceDiagram
  participant new
  participant set_ex
  participant get
  participant delete

  new ->> new: Generate client
  new ->> set_ex: client
  set_ex ->> set_ex: store key and value and genearte ttl

```

### ログイン機能の実装

#### 実装手順

`kernel → adapter → api`の順に実装を進める。

## テスト用の記述

ログイン機能

```zsh
curl -v "http://localhost:8080/auth/login" \
-H 'content-type: application/json' \
-d '{"email":"mebiusu1968@gmail.com", "password": "Swimming3003"}'
```

ログアウト機能

```zsh
curl -v -X POST "http://localhost:8080/auth/logout" \
-H 'Authorization: Bearer 要設定'
```

本の一覧取得

```zsh
curl -v POST "http://localhost:8080/api/v1/books" \
-H 'Authorization: Bearer c28decb21f7d49d296340d87f28fb8c2'
```

本の追加

```zsh
curl -v -X POST http://localhost:8080/api/v1/books \
-H 'Authorization: Bearer db15f79f8de94b8992c4f291c21fb3f9' \
-H 'Content-Type: application/json' \
-d '{"title":"","author":"","isbn":"","description":""}'
```


## 貸出機能のAPI一覧

|HTTPメソッド|パス|説明|Rustの関数名|
|------|---|---|---|
|POST|/api/v1/books/{book_id}/checkouts|蔵書の貸出操作を行う|checkout_book|
|PUT|/api/v1/books/{book_id}/checkouts/{checkout_id}/returned|蔵書を返却する|return_book|
|GET|/api/v1/books/checkouts|未返却の貸出中の蔵書一覧を取得する|show_checked_out_list|
|GET|/api/v1/books/{book_id}/checkout-history|該当蔵書の貸出履歴を取得する|checkout_history|
|GET|/api/v1/users/me/checkouts|自分が借りている蔵書の一覧を取得する||

### 実装の流れ

1. kernelでの型とRepositoryトレイトの追加
2. adapterでのトレイト実装
3. registryへの追加
4. apiでのAPI実装
