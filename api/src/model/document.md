# modelモジュール

## 責任範囲

WebAPIのリクエスとレスポンスでやり取りするデータ型を実装する

## 構成

* UserResponse: WebAPIで「ユーザー」の情報として返したい型
* UsersResponse: UserResponseを一覧で返す場合のデータ構造
* RoleName: UserResponseの中で扱うRoleの型
* ~~Request: APIリクエスト時のペイロードで受け取るデータ
* それぞれの型に対するFromトレイトの実装：kernelクレートで定義された同種の型に変更するための実装
