# n08 — `crates/proto-hub/src/error.rs`（35行・小さい）

## 役割

proto-hubが生成するエラーコードの一覧（設計決定D9: 「エラーは必ずcode+cause付き
1行」）。短いファイルなので数分で読み終わるが、**エラーコードの正はここ1箇所**
という原則を体現している重要なファイル。

## 読みどころ

- 3〜13行目 WS/adapter/keymap切替/Deck関連の基本コード
  （`WS_TOKEN_INVALID` `WS_PARSE` `ADAPTER_SENDINPUT_FAIL` `KEYMAP_SWITCH_UNKNOWN`
  `DECK_UNKNOWN_SLOT` `INTERNAL` `RELOAD_INVALID`）。
- 5〜6行目 `KEY_UNKNOWN_ID` / `KEY_RESOLVE_NONE` … **`proto_keymap`のものを
  再輸出（re-export）しているだけ**。定義はproto-keymap側にあり、ここでは
  `pub const X: &str = proto_keymap::X;`という形で「proto-hubのエラー一覧を見れば
  proto-keymap由来のものも含めて全部載っている」ようにしている。
- 17〜19行目 `LOAD_JSON_SYNTAX` / `LOAD_SCHEMA_INVALID` / `LOAD_VK_UNKNOWN` も同様の
  再輸出。コメントに「値は1箇所の文字列に一致」とある通り、文字列そのものの
  定義は1箇所だが、参照しやすいようにこちらにも定数として持たせている。
- 21〜35行目 P-005/P-007（レイアウト保存・レイヤー保存・盤面保存）で追加された
  比較的新しいコード。`_REJECTED`（検証に落ちた・形が不正）と`_FAILED`
  （ファイル操作そのものが失敗した）を分けているパターンに注目
  （`LAYOUT_SAVE_REJECTED`/`LAYOUT_SAVE_FAILED`、`LAYER_SAVE_*`、`BOARD_SAVE_*`）。

## なぜこうなっているか

- 「REJECTED（内容が悪い）」と「FAILED（操作自体が失敗した）」を分けているのは、
  ユーザーへの説明のしやすさのため。前者は「JSONを直せば直る」、後者は
  「ディスクやパーミッションの問題かもしれない」という別種の対処が要る。
- 1ファイルにコード一覧を集約しているので、ws.rsで新しいエラーを返したくなった
  ときに「まずここに定数を足す」という一貫した作法になっている
  （文字列リテラルを直接ばらまかない）。

## 理解度チェック

1. `KEY_UNKNOWN_ID`の本当の定義はどのファイルにあるか？
2. `_REJECTED`系と`_FAILED`系のエラーコードは、それぞれどんな状況で使い分ける
   べきか？
3. なぜエラーコードを文字列リテラルのまま各所に書かず、定数としてここに
   集約しているのか？
