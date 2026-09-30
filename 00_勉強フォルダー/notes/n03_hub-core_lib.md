# n03 — `crates/hub-core/src/lib.rs`（726行・vendored凍結）

## 役割

`CLAUDE.md`にある通り**1行も変更禁止**の凍結crate。別プロジェクト（Control Deck）
から持ち込まれた汎用ロジックで、proto-hubはこの中の`CommandService`（コマンドの
冪等・拒否判定＝設計決定D5）だけを借りて使っている（`proto-hub/Cargo.toml`の
依存コメント参照）。**このファイルは「読むだけ」でよく、変更する場面が来ることは
基本的にない。**

## 前提

- 1〜5行目のコメントにある通り、これは元々「View/transportに依存しないドメイン
  ロジック」として書かれた、このプロジェクト用ではない汎用コード。用語（Screen、
  Command、Inspector等）が proto-hub の語彙と少しズレて見えるのは、由来が違うため。
- 全部を理解する必要はない。**なぜ凍結されているか**と**どの部分が実際に使われて
  いるか**が分かれば十分。

## 読みどころ

- 11〜31行目 `ErrorCode` / `ScreenState` … 汎用のエラーコード・画面状態。
  proto-hub独自のエラーコードは`crates/proto-hub/src/error.rs`が別に持っている
  （こちらとは別物）。
- 66〜111行目 `StateSnapshot` / `HubState`（同名だが`proto-hub::state::HubState`とは
  **別の型**。名前が同じでも由来が違うので混同しないこと）。
- 113〜311行目 `CommandRegistry` / `CommandInvocation` / `CommandStatus` /
  `CommandResult` / `CommandOutcome` / **`CommandService`**（233行目）… ここが
  proto-hubから実際に借用されている中心部分。「同じコマンドを二重に実行しない
  （冪等）」「許可されていないコマンドは拒否する」というロジックがここにある。
- 312〜461行目 `InspectorValue` / `InspectorConstraint` / `InspectorField` /
  `InspectorTarget` / `InspectorStore` … 「Inspector契約」と呼ばれている部分
  （INDEX.mdに名前だけ出てくる）。今のproto-hubでは深く使われていない可能性が
  高いので、さらっと目を通す程度でよい。

## なぜこうなっているか

- CLAUDE.mdが「1行も変更禁止」としているのは、これが**他プロジェクトの正**であり、
  ここを直接書き換えると移植元との整合が壊れるため。機能を変えたい場合は
  proto-hub側で「借りた関数の外側」にロジックを足す、という設計になっている。
- 読む目的は「どこまでが借り物で、どこからがKeyDeck独自か」の境界線を掴むこと。
  この境界が分かると、`crates/proto-hub/src/state.rs`を読むときに「これは
  hub-coreから来た概念か、proto-hub独自か」を迷わず判別できる。

## 理解度チェック

1. `CommandService`の「冪等」とは、具体的にどういう状況を防ぐための性質か？
2. このファイルを直接編集してよい場面が来るとしたら、それはどんな異常事態か
   （＝普通は起きないと言えるか）？
3. `hub-core::HubState`と`proto-hub::state::HubState`が別物だと知らずにコードを
   読むと、どんな誤解が起きそうか？
