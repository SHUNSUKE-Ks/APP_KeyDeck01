# P-008 段階A: 端末スロット（端末ごとの盤面・押した端末だけ移る・to:"all"）

## 完成状態
- `devices/devices.json`（手書き・最大3台）を置くと、端末ごとの入口 URL（`/layout?device=<id>&id=<盤面>&token=`）が起動バナーに出る
- その URL から開いた端末は、**盤面の切り替えがその端末だけに届く**。`layout.switch` に `"to": "all"` を付けたボタンだけは全端末がそろって移る
- 「既定へ戻る」（id 省略）は、端末の defaultLayout → Hub 全体の既定 の順
- 未登録の `device=` を名乗った接続は WS の確立前に 403（`WS_DEVICE_UNKNOWN`）
- **devices.json が無ければ今の動きのまま**（端末を区別しない）。今回は見本 `devices.example.json` だけ置き、devices.json は置いていない（端末3台の実物が未回答のため）

## 変更内容
- 新規 `crates/proto-hub/src/device.rs`: 読み込みと検査（上限3・id `[a-z0-9_]{1,32}`・重複・空ラベル・未知の盤面・未知の欄を拒否）
- `proto-keymap`: `Action::LayoutSwitch` に `to: Option<SwitchScope>`（`"all"` のみ。省略した既存 JSON はそのまま読め、書き出しても欄は増えない）
- `state.rs`: 接続ごとの device、端末ごとの「いまの盤面」（メモリのみ）、`broadcast_to_device`、`connection_url("device:<id>")`
- `ws.rs`: `WsQuery` に device / layout、`check_device`、`layout.switch` の送り先の振り分け、`/api/schema` の説明に `to?`
- `main.rs`: devices.json の読み込み（盤面の後）、起動バナーに端末ごとの入口
- `static/layout.html`: `?device=` と今の盤面を WS の URL に載せる
- `devices/devices.example.json`（見本）、`できること.md` 版2.5（§4・§6・§7）、P-008 に裁定を追記

## 検証結果
- `cargo test --workspace` = **172 passed, 0 failed**（hub-core 7 + adapter 8 + proto-hub 109 + keymap 48。新規14件: device.rs 6件・keymap 2件・ws.rs 6件）。`cargo build` 警告0
- 本物の Hub（一時的に devices.json＝見本の写しを置いて起動）で:
  - 起動ログ `device slots loaded devices=3`、バナーに3台の入口 URL
  - PC のブラウザ2枚（ipad／android1 を名乗る、どちらも note_story）で、**android1 の「既定へ戻る」を実マウスでクリック → android1 だけ game_iphone7_port へ移り、ipad は note_story のまま**。Hub ログ `layout switch; this device only device="android1" sent=1`
  - `device=android9` の WS アップグレードは 403、`device=android2` は 101
  - 確認後、一時の devices.json は消した
- **未確認**: iPad / Android の実機。`to:"all"` は単体テストのみ（実物のボタンにはまだ付けていない）

## 次の行動
- 利用者から端末3台の実物（名前・機種・向き・最初の盤面）をもらい、devices.json を置く
- 段階B: `GET /api/devices` と、QR ギャラリー `/connect` を端末ごとに
- 統合編集画面（`/editor_v3`）の実装。表示レイアウトの保存先 `views/` を不変条件6へ追記してから
