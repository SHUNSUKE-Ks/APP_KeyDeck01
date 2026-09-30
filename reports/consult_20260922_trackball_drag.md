# 相談: スマホのトラックボールで「押したまま範囲を広げる」ができない

作成 2026-09-22 ／ 対象リポジトリ `C:\00_master\DevApps\APP_KeyDeck01`（独立git。GitHub: SHUNSUKE-Ks/APP_KeyDeck01）

---

## 1. 相談したいこと

Windows PC を、スマホ（iPhone 7・Safari）のブラウザ画面から操作する自作アプリ **KeyDeck** があります。
PC 側は Rust 製の Hub、スマホ側は素の HTML/JS の画面で、WebSocket でつながっています。

**Windows の範囲スクショ（Win+Shift+S）で、左ボタンを押したまま範囲をドラッグして囲む**ことを、スマホのトラックボール画面から行いたいのですが、実機で一度も成功していません。

次の3点について意見をください。

1. 失敗の原因はどこにありそうか（下の「確認済み」「未確認」を踏まえて）
2. 「押したまま動かす」を、スマホで確実にできる操作としてどう設計するのがよいか（下の案 A〜C の評価、または別案）
3. 選んだ案を、このリポジトリの制約（§6）を守って実装するなら、どのファイルをどう変えるか

---

## 2. 仕組みの概要（数行で）

```
スマホの画面（static/*.html）
  ├ キーの部品   … 押した「キーID」だけを Hub へ送る（key.press）。何が起きるかは Hub 側の JSON が決める
  └ トラックボール … 指の相対移動量（surface.state）と、ジェスチャー名（surface.gesture）を送る
                     ジェスチャー名の意味（クリック等）は Hub 側の surfaces/trackball.json が決める
Hub（Rust）
  └ 受け取った ID/名前を JSON で引いて Action にし、Windows の SendInput で実際のキー・マウス入力として送る
```

画面は「board（格子）」に「区画」を並べ、各区画に部品を1つ置く形です。トラックボールも1区画で、`/layout` の中に iframe で埋め込まれます。

---

## 3. 症状

- 手順: 範囲スクショのキー（Win+Shift+S）を押す → 画面が暗くなる → トラックボールで囲もうとする
- 期待: 左ボタンが押された状態でカーソルが動き、範囲が広がる
- 実際: 範囲が選べない（どう失敗しているか＝カーソルだけ動くのか、クリックで閉じるのか、はまだ実機で切り分けていない）
- 以前の試み: 別の区画に置いた「左クリック」キーを指で押さえたまま、トラックボールを動かした → 範囲は広がらなかった

---

## 4. 分かっていること

### 確認済み

- **Hub から Windows への送り方はドラッグとして正しい形**
  - 押す/離す: `SendInput` + `MOUSEEVENTF_LEFTDOWN` / `LEFTUP`（別々のジョブとして別タイミングで送る）
  - 移動: `SendInput` + `MOUSEEVENTF_MOVE`（相対移動。絶対座標や SetCursorPos ではない）
  - 押している間に Hub が勝手にボタンを離す処理は無い
- **今動いている Hub はトラックボールの押しっぱなし用ジェスチャー（`tapdrag1`）を読み込み済み**（Hub 起動 13:50 ＞ 設定変更 13:22）
- **「左クリック」キー（`mouse.click`）は、押した瞬間に down→up を一度に送る**。指で押さえ続けても押された状態にならない → §3 の「以前の試み」が失敗したのはこのため
- **キーの部品から「マウスボタンを押しっぱなし」を送る正式な手段は無い**。キーボードのキーには `key.hold`（押している間だけキーを押す）があるが、マウスボタン版は無い
- トラックボール画面（Vol2）の押しっぱなしの出し方: **タップ → 0.3秒以内にもう一度触れる → 0.2秒置く（または動かす）** で左ボタン down、指を離すと up
  - PC のブラウザ上で、実マウス操作と合成イベントで、送られるジェスチャーが期待どおりであることは確認済み（偽の WebSocket で記録）
  - **iPhone 実機では未確認**

### 未確認（ここが怪しい）

- 実機で `tapdrag1` の down/up が Hub に届いているか（Hub のコンソールに `surface gesture ... gesture_id=tapdrag1` の行が出るはず）
- スマホでの「タップ → 0.3秒以内に触り直す」が、人の操作として間に合っているか。間に合わないと、1回目がただのクリック（範囲スクショ画面ではクリックで何も選べない/閉じる可能性）になり、2回目はボタン無しのカーソル移動になる
- Windows 11 の範囲スクショ画面が、SendInput で注入されたマウスのドラッグを受け付けるか（一般には受け付けるはず、と考えているが実測していない）
- keymap JSON の型の上では `{"t":"mouse.button","button":"left","down":true}` を書けてしまう（serde の rename がある）。ただし `down` が固定値でキーの押す/離すと連動しないこと、許可リスト（`canonical_command_id`）での扱いは未確認。正式な使い方ではない

---

## 5. 検討中の案

| 案 | 中身 | 利点 | 気になる点 |
|---|---|---|---|
| **A. トラックボールに「つかむ」トグルを足す**（今の第一候補） | 球の画面に「✋」アイコンを重ねる。1回押すと left down（`tapdrag1` down を流用）、もう一度で up。その間は1本指で球を動かすとドラッグ | 間合い不要。片手・1本指で済む。Hub の変更不要 | 押したまま画面を閉じた/通信が切れたときにボタンが押されたまま残る → 切断時・非表示時に必ず up を送る処理が要る。Hub 側でも「クライアント切断時に押しっぱなしのマウスボタンを離す」安全策が要るか |
| **B. キーの部品に `mouse.hold` を新設**（`key.hold` のマウス版） | キーを押している間だけ left down。別区画のキーを押さえながら球を動かす | 既存の `key.hold` と同じ形で筋がよい | スマホで2本の指を別区画に同時に置く操作がしづらい（特に縦持ち片手）。iframe（球）と親ページ（キー）で同時タッチが正しく分かれるか要確認。Rust の Action 追加・許可リスト・`できること.md` の更新が要る |
| **C. 今の「タップ→押しっぱなし」の間合いを広げる** | 0.3秒 → 0.5秒など | 変更が小さい | 1回目のタップのクリック送信がそのぶん遅れる。根本的に間合い頼み |

---

## 6. 守るべき制約（このリポジトリの規則。`CLAUDE.md` より）

- **不変条件1**: 端末（スマホ）が送ってよいのは「位置ID」か「面が発する正規化状態（相対移動量など）」だけ。**任意のキー・任意のコマンドを送る WS API を作らない**。何が起きるかは必ず Hub 側の JSON が決める（許可リスト防御）
- **D28**: トラックボール等は `surface.state`（surfaceId＋±200 にクランプした dx/dy）を送ってよい。出口（binding）は Hub の JSON だけが決める
- **D2**: 端末は素の HTML+JS。フレームワーク・ビルドツール・CDN・PWA 禁止
- **D9**: エラーは code+cause の1行ログ。実行時入力で panic しない
- 凍結: `crates/hub-core/`（変更禁止）、`brief/` 配下の設計書（変更禁止）、`static/trackball_v1_1.html`（トラックボール Vol1.1 の凍結版）
- 機能を足したら同じ作業で `C:\00_CreatorCompass\KeyDeck\できること.md` を直す（他プロジェクトの AI がこれを見て判断するため）

---

## 7. 関係するファイル（絶対パス）

### 7.1 トラックボール（押しっぱなし・マウス機能の本体）

| ファイル | 何があるか |
|---|---|
| `C:\00_master\DevApps\APP_KeyDeck01\static\trackball.html` | **トラックボール Vol2（boardに埋め込まれる現行版）**。856行〜 `Gesture`（指の解釈。858行 `T_DTAP_MAX`、874行 `startHold`、958行 1回タップの確定）、711行〜 `hubSink`（WS送信。828行 `sendGesture`） |
| `C:\00_master\DevApps\APP_KeyDeck01\static\trackball_v1_1.html` | Vol1.1 凍結版（変更禁止）。2本指長押し＝範囲選択→離すと Ctrl+C、右の帯＝右クリック・スクロール |
| `C:\00_master\DevApps\APP_KeyDeck01\surfaces\trackball.json` | ジェスチャー名 → 意味の対応表（`tap1-left`＝左クリック、`dtap1`＝ダブルクリック、`tapdrag1`/`hold1`＝`mouse.button.hold` left 等） |
| `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-hub\src\surface.rs` | `surfaces/*.json` の読み込みと検証。79行 `GestureAction`、314行 `parse_gesture_wire` |
| `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-hub\src\ws.rs` | 1864行 `handle_surface_gesture`（ジェスチャー → Action → 送信）、1738行 `handle_surface_state`（相対移動）、1979行 `release_held_keys`（切断時に押しっぱなしのキーを離す。**マウスボタンは対象外かどうか要確認**） |
| `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-adapter-win\src\lib.rs` | Windows への実送出。205行〜 マウス移動・クリック・押す/離す（227行 `send_mouse_button`、298行 `mouse_button_down`、305行 `mouse_button_up`） |
| `C:\00_master\DevApps\APP_KeyDeck01\static\layout.html` | board を描く画面。570行 `renderTrackball`（トラックボールを iframe で埋め込む） |

### 7.2 ボタン（キーの部品）を作る仕組み

| ファイル | 何があるか |
|---|---|
| `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-keymap\src\lib.rs` | キーに書ける動作（Action）の型と読み込み検証。147行 `pub enum Action`、177行 `MouseClick`、183行 `MouseButton`（内部用）、196行 `KeyHold`（押している間だけキーを押す。案Bの手本） |
| `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-hub\src\state.rs` | 109行 `canonical_command_id`（許可リストのID）、347行 `note_key_hold` / 361行 `take_held_keys`（押しっぱなし中のキーの台帳。切断時に離すため） |
| `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-hub\src\ws.rs` | 1629行 `handle_key_press`（キーが押された/離されたときの処理）、301行 `schema_handler`（機械可読の動作一覧 `/api/schema`） |
| `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-hub\src\startup.rs` | 起動時に keymaps/decks/surfaces/layouts を全部読み、許可リストを作る |
| `C:\00_master\DevApps\APP_KeyDeck01\keymaps\keymap_click_left.json` と `keymaps\layers\click_left_layer0.json` | 1キー部品の実例（左クリック。`mouse.click` なので押しっぱなしにならない） |
| `C:\00_master\DevApps\APP_KeyDeck01\keymaps\keymap_screenshot_region.json` と `keymaps\layers\screenshot_region_layer0.json` | 範囲スクショ（Win+Shift+S） |
| `C:\00_master\DevApps\APP_KeyDeck01\schemas\keymap.schema.json` / `schemas\layer.schema.json` | キーマップ・レイヤーの JSON スキーマ |
| `C:\00_master\DevApps\APP_KeyDeck01\static\keys.html` | キー編集画面（ラベル・動作の割り当て） |
| `C:\00_master\DevApps\APP_KeyDeck01\static\editor.html` | board（区画の並べ方）の編集画面 |

### 7.3 いま使っている board

| ファイル | 中身 |
|---|---|
| `C:\00_master\DevApps\APP_KeyDeck01\layouts\layout_iphone7_portrait.json` | iPhone 7 縦・4×8。範囲スクショ・右クリック・トラックボール |
| `C:\00_master\DevApps\APP_KeyDeck01\layouts\layout_iphone7_youtube.json` | iPhone 7 横・10×4 |

### 7.4 規則・経緯・外部向け資料

| ファイル | 何があるか |
|---|---|
| `C:\00_master\DevApps\APP_KeyDeck01\CLAUDE.md` | AI 作業規則・不変条件・凍結領域（§6 の出典） |
| `C:\00_master\DevApps\APP_KeyDeck01\DEVBOARD.md` | 決定事項と検証記録の時系列。末尾に 2026-09-22 のトラックボール Vol2 の記録 |
| `C:\00_master\DevApps\APP_KeyDeck01\brief\keydeck_trackball_gestures_v0.7.md` | トラックボールのジェスチャー設計書（凍結・参照のみ） |
| `C:\00_CreatorCompass\KeyDeck\できること.md` | 他プロジェクト向けの「KeyDeck で何ができるか」。§4 に動作一覧とトラックボールのジェスチャー |

---

## 8. 回答してほしい形

- 原因の見立て（どれが一番怪しいか、実機でどう切り分けるか）
- 推す案と理由（A〜C か別案）。スマホ縦持ち・片手で使う前提で
- その案の変更点を、§7 のファイル単位で。§6 の制約に触れる箇所があれば明記
- 押しっぱなしが残ったまま（ボタンが離れない）になる事故への備え
