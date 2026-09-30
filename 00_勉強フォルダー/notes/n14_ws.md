# n14 — `crates/proto-hub/src/ws.rs`（2319行・最大のファイル。最後に読む）

## 役割

WSルーティング・全メッセージ処理・HTTP APIハンドラをまとめた、Hubの心臓部。
ここまで読んできたn02〜n13の全モジュール（proto-keymap・state・startup・
surface・deck・layout・protocol・error）が、実際に「1つのHTTPリクエスト/
WSメッセージの処理」としてどう組み合わさるかがこのファイルで見える。

## 前提

ここまでの全ファイルを読んでから取り組むこと。特にn02の`resolve()`、
n06の`HubState`、n07の`ClientMessage`/`ServerMessage`が分からないと
このファイルは読めない。分量が多いので、**1回で全部読もうとせず、
「1つのAPI/1つのメッセージ種別」ごとに区切って読む**のがおすすめ。

## 全体の地図（`router()`関数、40行目〜）

`.route(...)` の一覧がこのファイルの目次そのもの:

| パス | メソッド | ハンドラ | 何をするか |
|---|---|---|---|
| `/api/qr` | GET | `qr_image`（154行目） | n13のqr.rsを使ってQR画像を返す |
| `/api/reload` | POST | `reload_handler`（1030行目） | n09のstartup::load_startup_dataを再実行 |
| `/api/ping` | GET | `ping_handler`（206行目） | 疎通確認 |
| `/api/formats` | GET | `formats_handler`（272行目） | settings画面等が読む「今の構成」一覧 |
| `/api/hosts` `/api/host` | GET/POST | `hosts_handler` / `host_set_handler` | 接続先ホストの一覧・切替 |
| `/api/layout/save` | POST | `layout_save_handler`（390行目） | 不変条件6の`layouts/`保存 |
| `/api/layer/save` | POST | `layer_save_handler`（611行目） | 不変条件6の`keymaps/layers/`保存 |
| `/api/keymap/board/save` | POST | `board_save_handler`（787行目） | 不変条件6の`keymap_<id>.json`保存（boardのみ） |
| `/ws` | GET | `ws_handler`（176行目） | **WebSocket本体** |
| `/api/deck/export` | GET | `deck_export`（996行目） | Deckのエクスポート |

## 読みどころ（WS本体）

- 1099行目 `handle_socket(socket, state, surface)` … 接続確立後のループ。
  クライアントからのテキストメッセージを受け取るたびに`handle_client_text`を呼ぶ。
- 1144〜1178行目 `handle_client_text` … `ClientMessage`をパースし、4種類の
  メッセージ（`KeyPress` `DeckPress` `SurfaceState` `SurfaceGesture`）を
  それぞれの`handle_*`関数へディスパッチする、**このファイルの中心の中心**。
  パース失敗時は`WS_PARSE`エラーを返す（emit_error、1812行目）。
- 1185〜1219行目 `resolve_for_layout` / `resolve_for_surface` … 「どのkeymapの
  どのlayer_stateを使って解決するか」を、`surface`（Split/Ipad/Trackball/Layout）
  ごとに振り分ける関数。n06で見た「iPad面は独立したlayer_stateを持つ」が
  ここで実際に分岐している。**`SurfaceKind::Trackball`は`UnknownKey`を返す**
  （トラックボール面はkeymapを持たないため。防御的にpanicせず正常系のエラー値
  で応答する、というD9の実践例）。
- 1248〜1315行目 `handle_key_press` … `key.press`が来たときの一連の処理:
  1. ロックを取り、現在のレイヤーをログ用にスナップショット（1258行目）
  2. `resolve()`相当の関数を呼んで`Resolved`を得る
  3. **ログを必ず1行残す**（1266〜1279行目のコメントが秀逸。「押されたのに
     効かない」を追跡できる唯一の手がかりだと明言している。以前は失敗時
     しか記録しておらず、成功したのに実行結果が見えないケースを追えなかった、
     という改善の経緯）
  4. `Resolved`の種類ごとに応答: `UnknownKey`/`NoResolution`はエラー送信、
     `Ignored`は何もしない、`LayerChanged`はレイヤー状態を全体に配信、
     `Fire`は実際にアクションを発火、`FireAndLayerChanged`（tg.fire、T21）は
     **「先に配信→後に発火」の順序が固定**されている（1305〜1307行目の
     コメント: 発火はawaitを挟むので、先に画面を更新した方が体感が速く、
     発火が失敗しても状態の整合は崩れないため）。
- 1334〜1352行目 `handle_deck_press` … Deckのスロットを引いて発火するだけの
  シンプルな処理。`Action::None`は何もしない扱い。
- 1357行目〜 `handle_surface_state`（トラックボール等） … コメントに書かれた
  **処理順が5段階で固定**されている点が最重要:
  ① surfaceIdをレジストリで引く（無ければ`SURFACE_UNKNOWN_ID`）
  ② dx/dyが有限か確認（NaN/Infなら`SURFACE_STATE_RANGE`）
  ③ clampで範囲を絞る（n10のCLAMP値）
  ④ 丸め＋ゼロ移動はスキップ
  ⑤ bindingを解決して発火（既存の`adapter_tx`＝n06の直列ワーカーを使い、
     新しい発火経路を作らない）
  この順序が「なぜ固定なのか」は`brief/`のT12設計にあるが、**ここでは
  「後から順序を入れ替えると検証の意味が変わる」ということだけ覚えておけば十分**。
- 1470行目〜 `to_mouse_button_kind` … `surface.gesture`のbutton指定を
  proto-keymapの型へ変換する小さな橋渡し。
- 1619行目〜 `fire_action` … `Resolved::Fire`で得たActionを実際にadapter_txへ
  送る最終出口。ここから先はn04（proto-adapter-win）の世界。
- 1707行目〜 `switch_keymap` … `KeymapSwitch{id}`が発火したときの処理
  （アクティブキーマップの切替＋レイヤー状態リセット、n02の`LayerState::reset()`
  と対応）。
- 1848行目〜 `mod tests` … 1909行目以降のテスト名がそのまま「何を保証しているか」
  の一覧になっている（`g12a_unknown_surface_id_enqueues_no_job` =
  未知のsurfaceIdはジョブをキューに積まない、`g18a_hold1_without_edge_enqueues_no_job`
  = edge省略のhold系ジェスチャーは発火しない、等）。**コードを読む前にテスト名の
  一覧をざっと眺めると、このファイルが何を守っているかの見取り図になる。**

## セキュリティ系の読みどころ

- 171行目 `token_ok` … トークン検証。n06の`AccessToken::is_valid`（定数時間比較）
  を使っている実際の呼び出し箇所。
- 369行目 `layout_id_is_safe` … `CLAUDE.md`不変条件6の「layoutIdは`[a-z0-9_]{1,64}`
  のみ許可」を実装している**パスを組み立てる前の唯一の関門**。ここを緩めると
  任意パス書込になる、とCLAUDE.mdに明記されている箇所なので、実際にどんな
  正規表現/文字種チェックになっているか自分の目で確認しておく価値が高い。
- `layout_save_handler`（390行目）/ `layer_save_handler`（611行目）/
  `board_save_handler`（787行目） … 不変条件6の実装そのもの。共通パターンは
  「①IDの安全性チェック→②Layoutとして解釈し直す（生の本文をそのまま書かない）
  →③`.bak`へ退避→④書き込み→⑤全体を読み直して検証→⑥失敗したら巻き戻す」。
  3つのハンドラを読み比べて、この共通パターンがそれぞれどう繰り返されているか
  確認するとよい。

## なぜこうなっているか

- 1つのファイルにルーティングとハンドラを集約しているのは、
  「WSで何が起きるか」を1ファイルで完結して追えるようにするため
  （とはいえ2319行はさすがに大きく、n07のprotocol.rsやn06のstate.rsへ
  型・状態を切り出すことでこのファイル自体は「処理の流れ」に専念できている）。
- テスト名にgXXという番号が振られているのは、対応する設計書のチェックポイント
  （G12/G18等）と1対1で紐づけるため。「このテストが何を保証しているか」を
  探すとき、番号から`brief/`内の該当箇所を逆引きできる。

## 理解度チェック

1. `key.press`が`LayerChanged`と`Fire`と`FireAndLayerChanged`のどれに解決される
   かの違いを、n02の`resolve()`と結びつけて説明できるか？
2. `handle_surface_state`の5段階の処理順を、なぜこの順でなければならないか
   自分の言葉で言えるか（特に③clampが②有限性チェックの後である理由）？
3. `layout_id_is_safe`が無かったら、`layout_save_handler`はどんな攻撃に対して
   脆弱になるか？
