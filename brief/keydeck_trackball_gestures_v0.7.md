# トラックボール面 Ver1ジェスチャー拡張 v0.7（T15-T19）

> 前提: `brief/keydeck_trackball_design_v0.6.md`（T10-T14。実装済み・Vol凍結済み）の続き。
> D28の精神（クライアントは「何に繋がるか」を指定できない／出口はHub側JSONだけが決める）を
> そのまま踏襲する。新しい例外は作らない。

## 0. 対象範囲（ユーザー確定のVer1最小構成。2026-08-02）

| ジェスチャー | 機能 | 実現方式 |
|---|---|---|
| 1本ドラッグ | トラックボール操作（既存） | 変更なし |
| 1本タップ | 左クリック | §2 discrete gesture |
| 1本ダブルタップ | ダブルクリック | §2 discrete gesture |
| 長押し | マウス左ボタンを押しっぱなし（離すまでdown、離したらup） | §2 discrete gesture（edge付き） |
| 2本タップ | 右クリック | §2 discrete gesture |
| 2本上下 | スクロール | §3 continuous（D28 binding拡張） |
| 3本タップ | Esc | §2 discrete gesture（既存vk辞書のESCをそのまま使う） |

**Ver1では実装しない**（ユーザー裁定・2026-08-02）: 2本長押し＝リングメニュー。UIの中身（選択後の決定方法・項目内容）が未確定のため見送り。将来必要になったら別途設計する。

## 1. 全体設計方針

discrete（1回だけ発火／down-upで押しっぱなし）と continuous（連続値）で経路を分ける。

- **discrete**（タップ・ダブルタップ・長押し・Esc）→ 新しいWSメッセージ `surface.gesture`。
  クライアントは `surfaceId` + `gestureId` (+ 長押しのみ `edge`) だけを送る。**そのgestureIdが
  何をするかはクライアントのHTML/JSに一切書かない**。`surfaces/trackball.json` の
  `gestures` マップだけが意味を持つ（D5と同じ「位置ID→意味はHub側」の形をそのまま流用）。
- **continuous**（スクロール）→ 既存の `surface.state`（T12・D28）をそのまま使う。
  新しい `surfaceId`（`tb01-scroll`）を追加登録し、`binding.t` に新しい値
  `"mouse.scroll"` を許可リストへ追加するだけ。**新しいWSメッセージ型は不要**
  （同じWS接続で `surfaceId` を変えて送るだけでよい。`handle_surface_state` は
  `surfaceId` からレジストリを引くだけで、接続時の `surface=trackball` クエリとは
  無関係に動く＝既存コードの変更ゼロで対応可能）。

## 2. discrete: `surface.gesture` メッセージ

### 2.1 ワイヤー形式（`crates/proto-hub/src/protocol.rs`に追加）

```json
{ "type": "surface.gesture", "surfaceId": "tb01", "gestureId": "tap1" }
{ "type": "surface.gesture", "surfaceId": "tb01", "gestureId": "hold1", "edge": "down" }
{ "type": "surface.gesture", "surfaceId": "tb01", "gestureId": "hold1", "edge": "up" }
```

`edge`は省略可（`Option<EdgeWire>`。既存の`EdgeWire`をそのまま再利用）。
one-shot系（tap1/dtap1/tap2/tap3）はedgeを送らない。hold系（hold1）は必ず送る。

`ClientMessage`への追加（`key.press`/`surface.state`と並列）:
```rust
#[serde(rename = "surface.gesture")]
SurfaceGesture {
    #[serde(rename = "surfaceId")]
    surface_id: String,
    #[serde(rename = "gestureId")]
    gesture_id: String,
    #[serde(default)]
    edge: Option<EdgeWire>,
},
```

### 2.2 `surfaces/trackball.json` のスキーマ拡張（`crates/proto-hub/src/surface.rs`）

`SurfaceFileEntry`に**任意**フィールド`gestures`を追加（省略時は空マップ＝ジェスチャー無しの
面として従来どおり動く。既存の`tb01-scroll`用エントリなど無関係の面を壊さない）。

```json
{
  "surfaces": [
    {
      "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" }, "clamp": 200,
      "gestures": {
        "tap1":  { "t": "mouse.click",     "button": "left" },
        "dtap1": { "t": "mouse.dblclick",  "button": "left" },
        "tap2":  { "t": "mouse.click",     "button": "right" },
        "tap3":  { "t": "key",             "vk": "ESC" },
        "hold1": { "t": "mouse.button.hold", "button": "left" }
      }
    },
    {
      "id": "tb01-scroll", "type": "trackball", "binding": { "t": "mouse.scroll" }, "clamp": 100
    }
  ]
}
```

**この`gestures`の値はproto_keymap::Actionを再利用しない。** 独自の小さい列挙
`GestureAction`をsurface.rsに新設し、そこだけでパース・検証する（理由: `down`フィールドは
edgeから決まるものであって静的JSONに書く値ではないため、Actionをそのまま流用すると
「down/upどちらの意味か」を宣言時に決め打ちすることになり不自然。宣言＝「どのボタンか」
だけを持たせ、down/upはHub側が`edge`から組み立てる）。

```rust
// surface.rs に追加
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickButton { Left, Right }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GestureAction {
    Click { button: ClickButton },
    DoubleClick { button: ClickButton },
    ButtonHold { button: ClickButton },   // edge必須。Down→press、Up→release
    Key { vk: String },                   // is_known_vk()で既存vk辞書と同じ検証を通す
}
```

`SurfaceDef`に`pub gestures: BTreeMap<String, GestureAction>`を追加（デフォルト空）。

ロード時検証（新規エラーコード。`SurfaceError`と同じ形）:
- `t`が`"mouse.click"|"mouse.dblclick"|"mouse.button.hold"|"key"`以外 → `LOAD_SURFACE_GESTURE_INVALID`
- `button`が`"left"|"right"`以外 → `LOAD_SURFACE_GESTURE_INVALID`
- `t:"key"`の`vk`が`proto_keymap::is_known_vk()`を通らない → `LOAD_SURFACE_GESTURE_INVALID`
  （`LOAD_VK_UNKNOWN`は再利用しない。surface.rs系のエラーは`LOAD_SURFACE_*`で揃える）
- 同一面内で`gestureId`重複 → `LOAD_SURFACE_GESTURE_INVALID`

### 2.3 Hub側ハンドラ（`crates/proto-hub/src/ws.rs`）

`handle_client_text`のmatchに`ClientMessage::SurfaceGesture{..}`を追加し、新関数
`handle_surface_gesture`へ委譲する（`handle_surface_state`と並列の構造）。

```rust
async fn handle_surface_gesture(
    state: &SharedState, client_id: ClientId,
    surface_id: &str, gesture_id: &str, edge: Option<Edge>,
) {
    // ① surfaceId解決。無ければ既存のSURFACE_UNKNOWN_ID（surface.state と同じ意味なので使い回す）。
    // ② gestureId解決。無ければ新規 SURFACE_GESTURE_UNKNOWN_ID。
    // ③ GestureAction + edge から proto_keymap::Action を組み立てる:
    //    Click{button}      → 常に発火。 Action::MouseClick{button}
    //    DoubleClick{button}→ 常に発火。 Action::MouseDoubleClick{button}
    //    Key{vk}            → 常に発火。 Action::Key{vk}
    //      （Click/DoubleClick/Keyは edge==Some(Up) が来ても無視してよいが、
    //        本Volのクライアントはそもそもedgeを送らないため実質到達しない防御分岐でよい）
    //    ButtonHold{button} → edge必須。
    //        Some(Down) → Action::MouseButton{button, down:true}
    //        Some(Up)   → Action::MouseButton{button, down:false}
    //        None       → 新規エラー SURFACE_GESTURE_EDGE_REQUIRED（発火しない）
    // ④ 既存のadapter_tx（D7直列ワーカー）へAdapterJobとして送る。
    //    新しい発火経路は作らない。command_registry許可リストは通さない
    //    （T12のhandle_surface_stateがMouseMoveを直接adapter_txへ送る前例と同じ扱い。
    //     button/vkは起動時ロードのgesturesマップで既に固定されており、
    //     自由記述を受け付ける経路ではないため許可リストの対象外でよい）。
}
```

新規エラーコード（`surface.rs`に追加。既存の`SURFACE_UNKNOWN_ID`等と同じ場所）:
```rust
pub const SURFACE_GESTURE_UNKNOWN_ID: &str = "SURFACE_GESTURE_UNKNOWN_ID";
pub const SURFACE_GESTURE_EDGE_REQUIRED: &str = "SURFACE_GESTURE_EDGE_REQUIRED";
pub const LOAD_SURFACE_GESTURE_INVALID: &str = "LOAD_SURFACE_GESTURE_INVALID";
```

## 3. continuous: スクロール（`mouse.scroll` binding）

`crates/proto-hub/src/surface.rs`の`ALLOWED_BINDING_TYPES`に`"mouse.scroll"`を追加する
（今は`["mouse.move"]`のみ→`["mouse.move", "mouse.scroll"]`）。それ以外の`surfaces/*.json`
ロード経路・検証は無変更。

`crates/proto-hub/src/ws.rs`の`handle_surface_state`内、bindingを解決するmatch:
```rust
let action = match def.binding_t.as_str() {
    "mouse.move" => Action::MouseMove { dx, dy },
    "mouse.scroll" => Action::MouseScroll { dy },   // dxは無視してよい（横スクロールはVer1対象外）
    other => { /* 既存のINTERNAL防御分岐、変更なし */ }
};
```
`dx`は既存の①〜④の検証（有限性・clamp・丸め・ゼロスキップ）をそのまま通ってよい
（クライアント側は常に`dx:0`を送るので実害なし。Hub側に特別な分岐を増やさない）。

`static/trackball.html`側: 2本指の上下ドラッグを検出したら、既存の`hubSink`と同じ形の
WS送信をもう1本、`surfaceId: "tb01-scroll"`で行う（`delta.dx`は常に0、`delta.dy`だけ使う。
`spin`/`active`は`tb01`と同じ形式的な値をダミーで入れてよい＝Hubは読み捨てる）。

## 4. `proto_keymap::Action`への追加（`crates/proto-keymap/src/lib.rs`）

D28のMouseMove（T10）と同じ位置づけ＝「Hub内部が組み立てる、adapterへの具体的な命令」。
**通常のkeymap JSON（`keymaps/*.json`）からこれらを使うことは想定しないが、
MouseMove同様シリアライズ可能なAction列挙の一員である以上、型としては到達しうる。
existing precedent通り、それ自体は許容する（keymap JSONはHub運用者が管理する信頼済み設定
であり、D5の脅威モデルの対象外）。**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButtonKind { Left, Right }

// Action enumに追加:
#[serde(rename = "mouse.click")]
MouseClick { button: MouseButtonKind },
#[serde(rename = "mouse.dblclick")]
MouseDoubleClick { button: MouseButtonKind },
#[serde(rename = "mouse.button")]
MouseButton { button: MouseButtonKind, down: bool },
#[serde(rename = "mouse.scroll")]
MouseScroll { dy: i32 },
```

**注意（前回T10でSR-002が出た箇所。今回は事前に潰しておく）**: `Action`はexhaustive match
が複数箇所にある。新しい4variantを追加したら、以下を**全て**更新しないとコンパイルが
通らない。ここに列挙しておくので、コンパイルエラーで気づく前に自分から直すこと（SR不要）:

1. `crates/proto-keymap/src/lib.rs` の `validate_merged()` 内の大きなmatch
   （`Action::Trans | Action::None | ... | Action::MouseMove{..} => {}` の行に4つ追加）
2. `crates/proto-keymap/src/lib.rs` の `resolve()` 内、Fireの大きなmatch
   （`Action::Key{..} | ... | Action::MouseMove{..} => match edge {...}` の行に
   `MouseClick{..}`, `MouseDoubleClick{..}`, `MouseScroll{..}` を追加。
   **`MouseButton{..}`はここに入れない**——本Volのgestureは`resolve()`を経由せず
   `handle_surface_gesture`が直接Actionを組み立てるため、`resolve()`から見れば
   `MouseButton`は「resolve()自体には現れない」variant。ただしexhaustive matchの対象には
   なるので、Key等と同じdown-fire/up-ignoreのグループに入れて構わない
   （実害はない。到達しないコードパスがexhaustiveness目的で埋まるだけ））
3. `crates/proto-hub/src/deck.rs` の既存match
   （`Action::None | Action::KeymapSwitch{..} | ... | Action::MouseMove{..} => {}` の行）
   — **T10の時にここが漏れてSR-002になった。今回は最初から直す。**
4. `crates/proto-adapter-win/src/lib.rs` の `send()` 内match — §5参照。

## 5. `proto-adapter-win`（`crates/proto-adapter-win/src/lib.rs`）

`send()`に4つのアーム追加。既存の`MOUSEEVENTF_MOVE`と同じ`win`モジュール内に
`MOUSEEVENTF_LEFTDOWN/LEFTUP/RIGHTDOWN/RIGHTUP/WHEEL`を追加インポートする
（`windows`クレートの同じ`KeyboardAndMouse`モジュールに既にある定数）。

```rust
Action::MouseClick { button } => send_mouse_click(*button),
Action::MouseDoubleClick { button } => send_mouse_double_click(*button),
Action::MouseButton { button, down } => send_mouse_button(*button, *down),
Action::MouseScroll { dy } => send_mouse_scroll(*dy),
```

- `send_mouse_click`: down→up を1組（`MouseButtonKind::Left`→LEFTDOWN,LEFTUP／
  `Right`→RIGHTDOWN,RIGHTUP）。`send_key`と全く同じ形。
- `send_mouse_double_click`: click相当を2回連続で送るだけ（間隔調整は不要。
  SendInputは高速に送れるが、Windowsの二重クリック判定は「同位置・時間閾値以内」
  であって最短間隔の下限は無いため、connectedな4イベントで成立する）。
- `send_mouse_button`: `down==true`ならDOWNイベント1つだけ、`false`ならUPイベント1つだけ
  （down/upが別々のAdapterJobとして別タイミングで届く前提。中で対を作らない）。
- `send_mouse_scroll`: `MOUSEEVENTF_WHEEL`、`mouseData = -(dy * SCROLL_UNIT)`
  （`dy`は「指が下に動いた量」なので符号反転。`SCROLL_UNIT`は最初は`8`程度の固定値で仮置きし、
  実機テストで体感を見てから調整してよい——ここは正確性より「まず動く」を優先してよい箇所）。

非Windows(`cfg(not(windows))`)ダミー分岐にも同じ4関数を追加し、ログのみ・成功扱いにする
（既存の`mouse_move`ダミーと同じパターン）。

**実機での実際のクリック/ボタン送出テストは`examples/smoke_mouse.rs`と同じ理由で
自動テストから絶対に呼ばない。** 新しいexample（例: `examples/smoke_mouse_click.rs`）を
手動実行用に追加してよい。

## 6. `static/trackball.html` 側の変更

現状は単一ポインタのdrag専用（`pointerdown/pointermove/pointerup`が1本しか想定していない）。
複数指を扱うため、次の変更が要る:

- `pointerId`ごとに現在アクティブなポインタを`Map`で管理する。
- 判定ロジック（既存のCore/View/Sinkとは別の、新しい小さなモジュール`Gesture`として実装。
  Core/View自体は変更しない——移動量に関する既存のdrag経路は無変更のまま）:
  - 1本タップ: down→up が`T_TAP_MAX`(例: 250ms)以内、かつ移動量が`MOVE_THRESHOLD`px未満
    （このpxを超えたら「タップ未遂」でCoreの通常drag扱いに委ねる＝既存挙動を壊さない）。
  - ダブルタップ: 直前のタップ完了から`T_DTAP_MAX`(例: 300ms)以内に2回目のタップ完了。
  - 長押し: down継続が`T_HOLD`(例: 500ms)を超えて、かつその間の移動量が
    `MOVE_THRESHOLD`未満のまま。閾値到達した瞬間に`hold1`の`edge:"down"`を送る。
    finger up（またはcancel）で`edge:"up"`を送る。閾値未到達のままupしたら何も送らない
    （＝タップ判定に回る）。
  - 2本タップ: 2本目のpointerdownが1本目のdown中に発生し、両方とも上記タップ条件
    （時間・移動量）を満たしたままそろってupした場合に発火。
  - 2本上下（スクロール）: 2本とも接地したまま、平均移動量（2本のdyの平均）を
    §3の`tb01-scroll`へ送る。1本ドラッグ用のCoreは2本目が乗った時点で無効化する
    （＝1本ドラッグと2本操作が同時に混線しない）。
  - 3本タップ: 3本目版の2本タップと同型。
- 既存の1本ドラッグ（ボールの見た目・慣性）は**pointerが1本だけの間のみ**有効にする
  （2本目が触れた瞬間、現在進行中のCore dragは`Core.end()`相当で打ち切る）。
- 送信は`hubSink`と同じWebSocketインスタンスを共有してよい（新しい接続は作らない）。
  `surface.gesture`用の小さな送信関数を`hubSink`に1つ追加するだけでよい。

**この節はSonnetの裁量に委ねてよい範囲が広い**（具体的な閾値の数値・実装の細部）。
数値は仮決めで進めてよく、実機テストで調整する前提とする（既存のsensitivity等と同じ
`P`パラメータJSON方式に載せる必要はない——閾値はコード内固定値でよい。将来調整したくなったら
別途パラメータ化する）。

## 7. タスク分解（T15-T19）

| ID | 内容 | 触るファイル |
|---|---|---|
| T15 | `Action`に4variant追加＋exhaustive match 3箇所を直す（§4の1-3） | proto-keymap/src/lib.rs, proto-hub/src/deck.rs |
| T16 | `proto-adapter-win`に4関数追加（§5） | proto-adapter-win/src/lib.rs |
| T17 | `surface.rs`: `GestureAction`型・`gestures`マップのロード検証・新エラーコード（§2.2）／`ALLOWED_BINDING_TYPES`に`mouse.scroll`追加（§3） | proto-hub/src/surface.rs |
| T18 | `protocol.rs`に`SurfaceGesture`追加（§2.1）／`ws.rs`に`handle_surface_gesture`＋`handle_surface_state`のbinding match拡張（§2.3, §3） | proto-hub/src/protocol.rs, proto-hub/src/ws.rs |
| T19 | `surfaces/trackball.json`更新（§2.2の例をそのまま反映＋`tb01-scroll`追加）／`static/trackball.html`のジェスチャー検出実装（§6） | surfaces/trackball.json, static/trackball.html |

## 8. 受け入れ基準（G-15〜G-19）

- G-15a: `cargo test --workspace` 全pass（既存39件+から減らさない）
- G-15b: `proto-keymap`にT10と同型のserdeラウンドトリップ単体テストを4variant分追加
  （`{"t":"mouse.click","button":"left"}`等）
- G-16a: 非Windowsダミーで4関数とも成功を返す単体テスト（T10の`t10_dummy_backend_...`と同型）
- G-17a: 不正な`gestures`（未知の`t`／未知の`button`／未知の`vk`／重複gestureId）が
  それぞれ`LOAD_SURFACE_GESTURE_INVALID`で起動拒否されることを確認する単体テスト
- G-17b: `mouse.scroll`が`ALLOWED_BINDING_TYPES`に入り、`surfaces/trackball.json`の
  `tb01-scroll`が正常ロードされることを確認する単体テスト
- G-18a: `handle_surface_gesture`の単体テスト（T12の`handle_surface_state`テスト群と同型。
  実SendInputは呼ばずAdapterJobを横取りする）:
  - 未知surfaceId→ジョブなし
  - 未知gestureId→ジョブなし（SURFACE_GESTURE_UNKNOWN_ID）
  - `tap1`→`Action::MouseClick{button:Left}`のジョブ
  - `hold1`+`edge:down`→`Action::MouseButton{button:Left,down:true}`
  - `hold1`+`edge:up`→`Action::MouseButton{button:Left,down:false}`
  - `hold1`+edge省略→ジョブなし（SURFACE_GESTURE_EDGE_REQUIRED）
- G-18b: `handle_surface_state`で`surfaceId:"tb01-scroll"`, `binding.t:"mouse.scroll"`の時、
  `Action::MouseScroll{dy}`のジョブが載ることを確認する単体テスト
- G-19a: 実機（Android）で1本タップ→実際に左クリックが飛ぶ、長押し→ドラッグ選択が
  できる、2本タップ→右クリックメニューが出る、2本上下→実際にスクロールする、
  3本タップ→Escの効果（例: ダイアログが閉じる）が出る、を確認する
  （測定方法は`brief/keydeck_trackball_design_v0.6.md`に記録済みの「測定の落とし穴」
  ＝Browser-pane操作自体がカーソルを動かす、に注意。実機ブラウザで直接確認すること）

## 9. 触ってよいファイル／触ってはいけないファイル

**触ってよい**（本Volの範囲）:
- `crates/proto-keymap/src/lib.rs`
- `crates/proto-hub/src/deck.rs`
- `crates/proto-hub/src/surface.rs`
- `crates/proto-hub/src/protocol.rs`
- `crates/proto-hub/src/ws.rs`
- `crates/proto-adapter-win/src/lib.rs`
- `crates/proto-adapter-win/examples/`（新規smokeファイル追加のみ）
- `surfaces/trackball.json`
- `static/trackball.html`

**触らない**（brief/keydeck_design_v0.4.mdの既存禁止事項と同じ）:
- `crates/hub-core/`
- `brief/`配下の**他の**設計書・モック（本ファイル自体とVol0.6は対象外として既に確定済み）
- `keymaps/`（本Volはkeymapを一切使わない設計のため触る理由が無いはず）

## 10. わからないこと・判断に迷ったら

このドキュメントに書いていない仕様上の疑問が出たら、実装を止めて
`brief/spec_return_log.md`にSR起票すること（推測で進めない）。
