# KeyDeck 設計書 v0.6 — トラックボール面（D28）

策定: 2026-08-01 ／ 裁定: `brief/proposals/P-002_trackball_surface.md`（採択済み・ユーザー直裁定）
実装担当: **Sonnet**（本書の凍結仕様のみを実装する。解釈が要る箇所は`spec_return_log.md`へSR起票して停止）

## 0. ゴールと範囲

**ゴール**: Android実機でボールをドラッグすると、PCの実カーソルが対応方向へ動く。

**本Volの範囲は「マウス出口のみ」。** 3D角度・ゲームパッド・クリック・ホイールは**着手しない**。
`spin`・`active` は受信して**読み捨てる**（将来のためにプロトコルだけ先に確保する）。

## 1. 前提となる裁定（D28）

`CLAUDE.md` アーキ不変条件1へ追記済み。要点のみ再掲する。

- クライアントは `surfaceId` ＋ **上下限つき数値**のみ送る
- **出口(`binding`)はHub側のJSONだけが決める。** クライアントは binding を送れない・選べない
- `delta.dx`/`dy` は **±200** でクランプ。超過は拒否
- 送信レート上限あり（125Hz相当）

根拠: 一般のUSB HIDマウス／トラックボールと同一の形（相対移動量2つ）であるため。

## 2. 触ってよいファイル / 禁止ファイル

| 区分 | ファイル |
|---|---|
| **新規作成してよい** | `crates/proto-hub/src/surface.rs`、`surfaces/trackball.json`、`schemas/surface.schema.json`、`static/trackball.html`、`crates/proto-adapter-win/examples/smoke_mouse.rs` |
| **変更してよい** | `crates/proto-keymap/src/lib.rs`（Action追加のみ）、`crates/proto-adapter-win/src/lib.rs`、`crates/proto-hub/src/{main.rs,ws.rs,state.rs,protocol.rs,startup.rs}`、`DEVBOARD.md` |
| **変更禁止** | `crates/hub-core/`（1行も不可）、`keymaps/` 配下すべて、`decks/`、`static/{kb,deck,ipad,settings}.html`、`brief/` 配下の既存文書、既存テスト（削除・弱体化禁止） |

**既存の面（split/Deck/iPad）の挙動を1ミリも変えないこと。** 追加のみで達成できる設計にしてある。

## 3. データ形式（正はJSON）

### 3.1 `surfaces/trackball.json`（新規。出口の宣言）

```json
{
  "surfaces": [
    {
      "id": "tb01",
      "type": "trackball",
      "binding": { "t": "mouse.move" },
      "clamp": 200
    }
  ]
}
```

- `id` … クライアントが名乗る `surfaceId`。未知のidは拒否
- `type` … 現状 `"trackball"` のみ許可
- `binding.t` … **許可リストは現状 `"mouse.move"` の1種のみ**。他の値はロード時に拒否
- `clamp` … 省略時 200。1〜1000 の範囲外は拒否

**出口の差し替えは将来この`binding.t`を変えるだけで済むこと**が本設計の目的。
`static/trackball.html` 側には出口の知識を一切置かない。

### 3.2 WSクライアント→Hub メッセージ（新規1種）

```json
{
  "type": "surface.state",
  "surfaceId": "tb01",
  "delta":  { "dx": 12, "dy": -3 },
  "spin":   { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 },
  "active": true
}
```

- 既存の `ClientMessage` は `#[serde(tag = "type")]` なので、`SurfaceState` バリアントを足すだけでよい
- `spin`・`active` は**必須だがHubは使わない**。受け取って捨てる（将来の3D面が同じ形を使うため）
- `delta.dx`/`dy` は f64 で受けて Hub 側で丸め・クランプする

## 4. エラーコード（D9書式・追加分）

いずれも `[KD][ERR][コード] cause=…` の1行。**panicは禁止。** 不正入力は必ず握って返す。

| コード | 発生条件 |
|---|---|
| `SURFACE_UNKNOWN_ID` | 未登録の `surfaceId` が来た |
| `SURFACE_STATE_RANGE` | `dx`/`dy` が `clamp` を超えた、または非有限値(NaN/Inf) |
| `LOAD_SURFACE_SCHEMA_INVALID` | `surfaces/trackball.json` の形が不正 |
| `LOAD_SURFACE_BINDING_UNKNOWN` | `binding.t` が許可リスト外 |

起動時に `surfaces/` の検証が1件でも失敗したら、既存の起動拒否と**同じ経路**で全件printして終了する
（`startup::load_startup_data` に相乗りする）。

## 5. タスク分解

### T10 — `proto-adapter-win`: 相対マウス移動

1. `proto-keymap` の `Action` へ追加:
   ```rust
   #[serde(rename = "mouse.move")]
   MouseMove { dx: i32, dy: i32 },
   ```
2. `proto_adapter_win::send()` に分岐を追加。`SendInput` + `INPUT_MOUSE` + `MOUSEEVENTF_MOVE`
   （相対移動）。既存の `send_key` と同じ書式で `AdapterError` を返す
3. 失敗時は既存同様 `ADAPTER_SENDINPUT_FAIL` として呼び出し側が整形できる形にする

**注意（既知の落とし穴）**: `SendInput` は**実カーソルを本当に動かす**。
**自動テストから絶対に呼ばないこと。** 自動テストの対象は
「`Action::MouseMove` のserde往復」「クランプ関数」など純粋な部分に限る。
実際の移動確認は手動smokeで行う: `crates/proto-adapter-win/examples/smoke_mouse.rs` を新規作成し、
`cargo run -p proto-adapter-win --example smoke_mouse` で数回だけ小さく動かす。

**受け入れ基準**
- G-10a `Action::MouseMove{dx,dy}` が `{"t":"mouse.move","dx":12,"dy":-3}` とserde往復する
- G-10b `send()` が `MouseMove` を受けてコンパイル・実行でき、smokeでカーソルが動く（手動）
- G-10c 既存のkey/chord/textの挙動とテストは無変更

### T11 — `proto-hub`: surfaces設定のロードと検証

1. `crates/proto-hub/src/surface.rs` を新規作成。`surfaces/trackball.json` を読み、
   `SurfaceRegistry { id -> SurfaceDef { binding, clamp } }` を構築する
2. `startup::load_startup_data` から呼び、既存のkeymap/deck検証と**同じエラー集約経路**に乗せる
3. `binding.t` の許可リストは**コード内の固定リスト**（現状 `"mouse.move"` のみ）。
   ここを緩めてはならない（D28の要件）
4. `surfaces/` ディレクトリまたはファイルが存在しない場合は**空レジストリで正常起動**する
   （既存ユーザーの起動を壊さないため）

**受け入れ基準**
- G-11a 正常なJSONでレジストリが構築される（単体テスト）
- G-11b `binding.t` が未知の値だと `LOAD_SURFACE_BINDING_UNKNOWN` で起動拒否（単体テスト）
- G-11c 形が壊れていると `LOAD_SURFACE_SCHEMA_INVALID` で起動拒否（単体テスト）
- G-11d ファイル不在でも起動は成功し、既存機能は無影響（単体テスト）

### T12 — `proto-hub`: `surface.state` の受信と発火

1. `SurfaceKind` に `Trackball` を追加。`from_query` で `Some("trackball")` を対応させる
   （**既存の `_ => Split` のフォールバックを壊さないこと**）
2. `ClientMessage` に `SurfaceState` を追加
3. `ws.rs` に `handle_surface_state` を追加。処理順は必ずこの順:
   1. `surfaceId` をレジストリで引く。無ければ `SURFACE_UNKNOWN_ID` を返して終了
   2. `dx`/`dy` の有限性を確認。NaN/Infなら `SURFACE_STATE_RANGE`
   3. `clamp` 超過なら `SURFACE_STATE_RANGE`（**握りつぶさずクライアントへerrorを返す**）
   4. 丸めて `i32` にする。`dx==0 && dy==0` なら**何も発火せず終了**（無駄なSendInputを打たない）
   5. `binding` を解決して `Action::MouseMove{dx,dy}` を作り、
      **既存の `adapter_tx`（D7の直列ワーカー）へ流す**。新しい発火経路を作らないこと
4. ルート `/trackball` を `ServeFile::new("static/trackball.html")` で追加
5. 起動時のURL一覧printに trackball の行を追加。QRターゲットにも `trackball` を追加

**受け入れ基準**
- G-12a 未知の `surfaceId` で `SURFACE_UNKNOWN_ID` が返り、Hubは落ちない
- G-12b `clamp` 超過・NaNで `SURFACE_STATE_RANGE` が返り、Hubは落ちない
- G-12c `dx=0,dy=0` ではAdapterJobが発行されない
- G-12d 正常値で `Action::MouseMove` が既存 `adapter_tx` に載る
- G-12e **split/Deck/iPadの既存WS挙動が無変更**（既存テスト全passで確認）

### T13 — `static/trackball.html`

`（scratchpad）trackball_canvas2d.html` を移植する。**モックが見た目・挙動の正。**
勝手なCSS追加・色変更・レイアウト変更をしないこと（過去にSonnetが`body.layer-active`を
発明して差し戻した事例がある）。

移植時の変更点は次の3点**のみ**:

1. WS接続を `ipad.html` と同じ規則にする: `/ws?token=…&surface=trackball`。
   `?ws=` クエリで手動指定する仕組みは削除してよい
2. ヘッダを他面と揃える: QRボタン(D25)・接続状態ボタン(D26)。`ipad.html` のCSSと構造を踏襲
3. エラーは必ず `console.error` に `[KD][ERR][コード]` 書式で出す（D9）

**保持すること（消さない）**:
- パラメータJSON（`<script type="application/json" id="paramSpec">`）とUI自動生成
- 右上⚙のパラメータ表示/非表示
- Core / View / Sink の3層構造。**Core・Viewに "mouse"/"マウス" を出さない**
- Canvas 2Dのみ。**Three.js等の外部ライブラリ・CDNは禁止（D2）**

**受け入れ基準**
- G-13a Androidの実機でボールをドラッグすると、PCの実カーソルが対応方向へ動く（最終確認）
- G-13b Core層・View層のコードに `mouse`/`マウス` が出現しない（分離の機械的確認）
- G-13c 外部リソースへのリクエストが0件（DevToolsのNetworkで確認。D2）
- G-13d 狭い画面幅（412px相当）で横スクロールが発生しない
  （`document.body.scrollWidth === window.innerWidth`。過去バグPF参照）

### T14 — 面のUI状態（追加仕様・2026-08-01 実機テスト後にユーザー要望で追加）

面は次の**2値の状態**を持つ。

| 状態 | 意味 |
|---|---|
| `settings` | 調整パネルを開いている |
| `trackball` | 通常のボール操作 |

- `settings` のとき、**パネルの枠の外をタップしたら `trackball` へ戻る**
- ただし**その「閉じるためのタップ」でカーソルを動かしてはならない。**
  閉じる動作だけで終わらせる（captureフェーズで捕まえて後続へ伝えない）
- パネル自身と⚙ボタンの上は「外」ではない（⚙は自前のclickで開閉する）

**受け入れ基準**
- G-14a パネル内をタップしても閉じない
- G-14b 枠外をタップすると閉じて `trackball` へ戻る
- G-14c その閉じるタップでカーソル移動が発火しない
- G-14d 閉じた後の通常ドラッグは従来どおり効く
- G-14e ⚙で開き直せる

## 6. 全体の品質ゲート

- `cargo test --workspace` 全pass。**既存54件を下回らないこと**（削除・弱体化は差し戻し）
- 既存動作の回帰確認: split左右のレイヤー同期／Deck発火／QR表示／iPad面
- 完了時に `DEVBOARD.md` の検証記録へ実行コマンドと結果を1行追記
- 大きめの変更のため、完了後に `keydeck-guardian` 点検を通すこと

## 7. Sonnetへ

- **本書の凍結仕様のみを実装すること。** 良かれと思った追加機能・CSS変更・リファクタは行わない
- 仕様に書かれていない判断が必要になったら、**推測で埋めずに** `brief/spec_return_log.md` へ
  SRを起票して停止し、報告すること
- T10 → T11 → T12 → T13 の順で進める（後段が前段に依存するため）
- 各Tの完了時に受け入れ基準への合否を明示して報告すること。
  **「たぶん動く」ではなく、実行したコマンドと出力で示すこと**
