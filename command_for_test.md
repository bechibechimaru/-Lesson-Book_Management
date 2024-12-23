# テスト用の記述

ログイン機能

```zsh
curl -v "http://localhost:8080/auth/login" \
-H 'content-type: application/json' \
-d '{"email":"input your email", "password": "input your pass"}'
```

ログアウト機能

```zsh
curl -v -X POST "http://localhost:8080/auth/logout" \
-H 'Authorization: Bearer input your user_token'
```

本の一覧取得

```zsh
curl -v POST "http://localhost:8080/api/v1/books" \
-H 'authorization: Bearer input your user_token' | jq .
```

本の追加

```zsh
curl -v -X POST http://localhost:8080/api/v1/books \
-H 'Authorization: Bearer input your user_token' | jq .
-H 'Content-Type: application/json' \
-d '{"title":"book1","author":"yamada","isbn":"eovheohoaehvae","description":"good book"}'
```

貸出の実行

```zsh
curl -v -X POST "http://localhost:8080/api/v1/books/ input book_id /checkouts" \
-H 'authorization: Bearer input your user_token'
```

返却の実行

```zsh
curl -v -X PUT "http://localhost:8080/api/v1/books/ input book_id /checkouts/ input Rental_ID /returned" \
-H 'authorization: Bearer input yout user_token'
```

貸出中の蔵書一覧取得

```zsh
curl -v "http://localhost:8080/api/v1/books/checkouts" \
-H 'Authorization: Bearer input your user_token ' | jq .
```
