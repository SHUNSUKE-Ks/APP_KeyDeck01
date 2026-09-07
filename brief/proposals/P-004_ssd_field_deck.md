# P-004 SSD外出先専用キーボードView（共通／ComfyUI／Ollama）

起票: 2026-09-05 ／ 起票者: KeyDeck相談セッション（ユーザー裁定を受けて作成）
種別: **製作依頼**（KeyDeck専属チャット宛て）
関連: `P-001_comfyui_control_panel.md`（本件はP-001の第1Volに相当）／`P-003_streamdeck_v2.md`（面の見た目）

---

## 0. この文書の使い方

**KeyDeck専属チャットは、着手前に §1 → §2 → §7 の順で読むこと。**
§2は実コードを読んで確認した「できること／できないこと」の一覧で、**ここを読まずに設計すると実装不可能な盤面を描くことになる**。

---

## 1. 依頼の背景とユーザー裁定（済）

外出先（マンガ喫茶等のレンタルPC。RTX4080搭載機を想定）にSSDを持ち込んで作業する際、
iPadをKeyDeckの面として使い、SSD作業（ComfyUI／Ollama／環境の初期準備）の定型操作をボタン化して
業務効率化する。

2026-09-05のユーザー裁定（このまま前提として扱ってよい。再確認不要）:

| # | 論点 | 裁定 |
|---|---|---|
| 1 | 開発を分岐（フォーク／別リポ）するか | **分岐しない。本流（main）に面とJSONを足す** |
| 2 | コマンド入力の自由度 | **事前登録した定型のみ**。iPadは押すだけ。任意コマンド実行APIは作らない（CLAUDE.md不変条件6を維持） |
| 3 | Operations Boardとの接続経路 | **Hubが中継する**。Boardは`127.0.0.1`のまま。Boardを`0.0.0.0`に公開しない |
| 4 | 盤面の区分 | **共通（初期準備を含む）／ComfyUI／Ollama** の3区分 |

現状すでにDeckでボリューム上下・戻る等のSteam Deck的操作はできている。今回はその上に
「SSD作業専用のView」を足すのが目的。

---

## 2. 【最重要】実装済み／未実装の事実（コードを読んで確認済み）

**この節は調査結果であり、推測ではない。設計はこの制約の内側で行うこと。**

### 2.1 そのまま使えるもの（新規コード不要）

- `{"t":"text","string":"..."}`（D20・KEYEVENTF_UNICODE）は実装済み。**バッチのパスやコマンド文字列をそのままPCへ流し込める**
- `{"t":"key","vk":"..."}` / `{"t":"chord","keys":[...]}` は実装済み
- `{"t":"mo","layer":N}` / `{"t":"tg","layer":N}` によるレイヤー切替は実装済み（D3の意味論）
- `keymaps/keymap_*.json` は**ディレクトリスキャンで自動発見**される（`crates/proto-hub/src/startup.rs:30` `discover_keymap_paths`）。ファイルを置いて`/settings`の再読込でよい

### 2.2 できないこと（ここを踏むと詰む）

| # | 事実 | 根拠（実ファイル） |
|---|---|---|
| A | **`/ipad`面のkeymapIdは`ipad01_vol12`にハードコードされている**。`keymap.switch`の影響を受けず、他の`kind:"single"`キーマップを表示する手段が現状ない | `crates/proto-hub/src/state.rs:21` `pub const IPAD_KEYMAP_ID: &str = "ipad01_vol12";` ／ `startup.rs:81`（未ロードなら起動拒否）／ `ws.rs:328`付近のコメント |
| B | **Deckのセットリストファイルは1つだけ**（`decks/deck_default.json`固定）。複数Deckファイルの切替機構はない | `crates/proto-hub/src/main.rs:24` `const DECK_PATH: &str = "decks/deck_default.json";` |
| C | **`deck.page`（Deckのページ切替）は未実装**。スキーマ上`pages`は配列だが、切替アクションが無いので実質1ページ | P-003 §11でVer1-b候補として未裁定のまま |
| D | Deckのgrid上限は **8列×6行＝48スロット** | `schemas/deck.schema.json`（cols max 8 / rows max 6） |
| E | `keymap.switch`が効くのは**split面とDeck面が共有する`active_keymap_id`**のみ。現存の切替可能キーマップは`default`(split)・`writing01`(split)の2つ | `ws.rs:671,749` `switch_keymap` |
| F | Hubに**HTTPクライアント依存が無い**（`reqwest`等が`Cargo.toml`に無い）。Board中継には依存追加が要る | `crates/proto-hub/Cargo.toml` |

### 2.3 したがって、盤面の置き場所は3択になる

| 案 | 内容 | コスト | 評価 |
|---|---|---|---|
| **案1（推奨）** | **`/ipad`面のkeymapId固定を解除**し、`/ipad?keymap=<id>`等で表示するキーマップを選べるようにする。その上で新規`kind:"single"`キーマップ`ssdfield01`を3レイヤー構成で作る | Rust小規模（`state.rs`・`ws.rs`・`qr.rs`／`static/ipad.html`）＋JSON | 3区分＝3レイヤーが素直に実現でき、**今後の全「専用View」（writing用・Gambit用等）が同じ仕組みで増やせる**。事実Aは T8 の設計判断に由来するので、**変更前に必ずSR起票してユーザー裁定を取ること** |
| 案2 | `decks/deck_default.json`に SSD用スロットを追記（48枠を行で3区分に割る） | **コード0** | 即日できるが、48枠に既存のメディアキー等と同居させることになり、区分の視認性が落ちる。事実C（ページ切替なし）が効く |
| 案3 | 新規`kind:"split"`キーマップ`ssdfield01`を作り、Deckの`keymap.switch`ボタンから切り替える | **コード0** | 切替もレイヤーも既存機構で動く。ただしsplit面は左右を別端末で開く前提のため、**iPad1枚では半分しか見えない**。要確認事項（§8-3） |

**依頼としては案1を本命とし、案1の裁定が下りるまでの間に案2または案3で「先に使える状態」を作ることを推奨する。**
どの案を採るかは専属チャットが§8の要確認事項を潰したうえで提案し、ユーザー裁定を取ること。

---

## 3. 依頼範囲（段階分け）

### 段階A：SSD外出先専用キーボードView（今回の主目的）

- A-1: §2.3の案の選定 → SR起票 → ユーザー裁定
- A-2: キーマップJSON（またはDeck JSON）の作成。中身は§4
- A-3: 案1を採る場合のRust実装（`/ipad`のkeymapId可変化）
- A-4: ポータブル版（`D:\00_WorkSpace\APP_KeyDeck01_portable\`）への反映手順の追記

### 段階B：Operations Board連携（手順書のiPad表示）

- B-1: Hubに中継エンドポイントを1本追加。**転送先はJSONで事前登録した固定IDのみ**（許可リスト）。`reqwest`依存の追加が発生
- B-2: 新面`static/board.html`（素のHTML+JS。D2遵守＝フレームワーク・CDN禁止）
- B-3: 表示は**まず等幅の素テキストで出す**。Markdown描画ライブラリは入れない（D2）。物足りなければ見出し・箇条書きだけの自前簡易整形を後から足す

Board側APIの調査結果（実コード確認済み）:

| エンドポイント | 返すもの |
|---|---|
| `GET /api/comfyui-board-docs` | 資料の一覧（id/category/title/path/available）。**本文は含まない** |
| `GET /api/comfyui-board-docs?id=<id>` | `{"id","category","title","path","content"}`。**contentはMarkdown生テキスト** |
| `GET /api/comfyui-catalog` | Model台帳・Workflow台帳 |
| `GET /api/comfyui-requests` | 依頼一覧 |
| `GET /api/comfyui-daily-tasks` | 今日のタスク |
| `POST /api/comfyui-requests/status` 他 | **ファイルを書き換える**。段階Cまで中継しないこと |
| `GET /__shutdown` | **認証なしでBoardサーバーを停止する。許可リストに絶対に入れないこと** |

### 段階C：書き込み系（依頼ステータス更新など）

今回は範囲外。P-001の「最初のVolは小さく切る」推奨に従う。

---

## 4. 盤面の中身（実在確認済みのパス／コマンド）

**下表の「確認」列が✅のものは2026-09-05にファイル実在をこのセッションが確認した。要確認のものは専属チャットが確認するまでボタン化しないこと。**

### 4.1 共通レイヤー（layer 0）

初期準備は「その日に1回しか押さない」ボタン群なので、**レイヤーを分けずに共通レイヤーの上段1行にまとめる**ことを推奨する（レイヤーを増やすと毎回の切替コストが上がるため）。

| ラベル案 | action | 値 | 確認 |
|---|---|---|---|
| ComfyUI起動(GPU) | text | `D:\00_START_ComfyUI_GPU.bat` | ✅ |
| ComfyUI起動(venv) | text | `D:\start_comfy_v2.bat` | ✅ |
| ComfyUI(CPU) | text | `D:\Run_CPU.bat` | ✅ |
| Board起動 | text | `D:\00_START_OperationsBoard.bat` | ✅ |
| Board停止 | text | `D:\00_STOP_OperationsBoard.bat` | ✅ |
| AI Memory開く | text | `D:\00_OPEN_AI_MEMORY.bat` | ✅ |
| PC監査 | text | `D:\00_RUN_PC_AUDIT.bat` | ✅ |
| Portable Setup | text | `D:\00_SETUP_ComfyUI_PORTABLE.bat` | ✅ |
| Claude起動(Git付) | text | `D:\00_START_Claude_withGit.bat` | ✅ |
| テストセッション | text | `D:\00_RUN_TEST_SESSION.bat` | ✅ |
| **実行** | key | `ENTER` | — |
| 中断 | chord | `["CTRL","C"]` | — |
| SSDへ移動 | text | `cd /d D:\` | — |
| ComfyUIへ | text | `D:\ai\ComfyUI` | ✅（実在確認済み） |
| L1: ComfyUI | tg | layer 1 | — |
| L2: Ollama | tg | layer 2 | — |

**設計上の要点**: `text`アクションはEnterを押さない。**「文字を流す」と「実行する」を必ず別ボタンに分けること。**
これは事故防止として重要で、ユーザー裁定2（事前登録の定型のみ）とも整合する。

### 4.2 ComfyUIレイヤー（layer 1）

| ラベル案 | action | 値 | 確認 |
|---|---|---|---|
| ComfyUI URL | text | `http://127.0.0.1:8188` | ✅（`00_AI_Memory/01_CURRENT_STATE.md:15`に記載） |
| Board URL | text | `http://127.0.0.1:8765/applications/comfyui-inbox/index.html` | ✅（`00_START_OperationsBoard.bat`内のURL） |
| output開く | text | `D:\ai\ComfyUI\output` | ✅（実在確認済み） |
| models開く | text | `D:\ai\ComfyUI\models` | ✅（実在確認済み） |
| — | — | ComfyUI画面内のショートカット（Queue Prompt等） | **要確認**。推測でvkを置かないこと |

### 4.3 Ollamaレイヤー（layer 2）

**⚠ 前提の確認が必要**: 2026-09-05時点で、この家PCには **Ollamaの実行ファイルが見つからなかった**
（PATH上に`ollama`なし、`C:\Users\enjoy\AppData\Local\Programs\`配下にもなし）。
Ollamaは外出先のRTX4080機側にある可能性が高い。**専属チャットは、どのPCでOllamaが動くのかをユーザーに確認してから作ること。**

以下のコマンドは `D:\ai\01‗ConfyUI_WorkSpace\Report\05_Ollama_qwen3_8b_起動確認_20260722.md` に**実際に記載されている**もの。

| ラベル案 | action | 値 | 確認 |
|---|---|---|---|
| qwen3:8b 起動 | text | `ollama run qwen3:8b` | ✅（同レポートに記載） |
| 稼働確認 | text | `ollama ps` | ✅（同レポートに記載） |
| qwen3:8b 停止 | text | `ollama stop qwen3:8b` | ✅（同レポートに記載） |
| モデル一覧 | text | `ollama list` | 要確認（一般的なコマンドだが同レポートに明記なし） |
| モデル保存先 | text | `OLLAMA_MODELS` の値 | **要確認**（同レポート52行目に「別の保存先を指定できる」とあるが、実際の設定値は未確認） |

**運用上の重要事項**（同レポートより）: 起動中のOllamaモデルはVRAMを使うため、**ComfyUI生成前に`ollama stop`でVRAMを解放する**運用になっている。共通レイヤーからも`ollama stop`が押せる位置に置くことを検討すること。

---

## 5. 必要資料のパス一覧

### 5.1 KeyDeck側（`C:\00_master\DevApps\APP_KeyDeck01\`）

| パス | 何のために読むか |
|---|---|
| `CLAUDE.md` | **必読**。凍結領域・不変条件6箇条。特に不変条件1（許可リスト）・6（任意パス書込/任意コマンド実行API禁止）・D2（フレームワーク/CDN禁止） |
| `INDEX.md` | 全ファイルの地図 |
| `STATE.md` | 現在地。**未コミット項目の一覧がここにある**（§7参照） |
| `DEVBOARD.md` | 決定事項ログ・検証記録。作業完了時はここに1行追記する |
| `brief/proposals/P-001_comfyui_control_panel.md` | 本件の元提案。ポータブル配布・ComfyUI連携actionの設計 |
| `brief/proposals/P-003_streamdeck_v2.md` | Deck v2の見た目と、`app.launch`等の未裁定アクション |
| `brief/keydeck_design_v0.4.md` | D18〜D27。Vol管理・text注入(D20)・VIAL設定(D22) |
| `schemas/keymap.schema.json` | **actionの正**。`$defs/action`のoneOfが使える全アクション |
| `schemas/deck.schema.json` | Deckの正。grid上限8×6 |
| `keymaps/keymap_ipad01_vol12.json` | `kind:"single"`のboard記法の見本（13列グリッド） |
| `keymaps/layers/ipad01_vol12_layer*.json` | レイヤーファイルの書式見本 |
| `decks/deck_default.json` | Deckの書式見本（`vk`名の実例＝MUTE/VOL_UP/MEDIA_NEXT等） |
| `crates/proto-hub/src/state.rs` | `IPAD_KEYMAP_ID`（事実A） |
| `crates/proto-hub/src/main.rs` | `DECK_PATH`（事実B） |
| `crates/proto-hub/src/startup.rs` | 自動発見・起動時検証 |
| `crates/proto-hub/src/ws.rs` | `switch_keymap`・reload |
| `reports/handoff_2026-09-05_portable_release_v1.md` | ポータブル版の中身と外出先での手順 |
| `.claude/agents/keydeck-guardian.md` | 大きめの変更後に通す守護レビュー |

### 5.2 SSD側（`D:\`）

| パス | 中身 |
|---|---|
| `D:\ai\01‗ConfyUI_WorkSpace\50_batch\tools\comfyui_board_server.py` | **Operations Board本体**（Python）。段階BのAPI仕様の正 |
| `D:\MyTools\debugTool\applications\comfyui-inbox\index.html` | Boardの画面。Boardサーバーが配信しているHTML資産 |
| `D:\00_START_OperationsBoard.bat` ／ `D:\00_STOP_OperationsBoard.bat` | Board起動・停止 |
| `D:\ai\01‗ConfyUI_WorkSpace\00_AI_Memory\DESIGN_OperationsBoard統合方針_20260731.md` | Board実装が3つ併存している経緯と方針 |
| `D:\ai\01‗ConfyUI_WorkSpace\00_AI_Memory\01_CURRENT_STATE.md` | ComfyUI URL等の環境情報 |
| `D:\ai\01‗ConfyUI_WorkSpace\00_AI_Memory\HANDOVER_NewChat_ComfyUI_2026-07-25.md` | ComfyUI側の入口ドキュメント |
| `D:\ai\01‗ConfyUI_WorkSpace\Report\05_Ollama_qwen3_8b_起動確認_20260722.md` | **Ollamaコマンドの一次情報** |
| `D:\ai\01‗ConfyUI_WorkSpace\10_manual\20260731_FieldTest\KeyDeck_iPad接続手順.md` | 前回の外出先でのiPad接続手順 |
| `D:\00_WorkSpace\APP_KeyDeck01_portable\` | ポータブル版の実体（2026-09-05作成・別PCでの起動は未検証） |

---

## 6. 受け入れ基準（G形式）

- **G-a**: iPadで新しいSSD専用Viewを開くと、共通／ComfyUI／Ollamaの3区分が切り替えられる
- **G-b**: 共通レイヤーの「ComfyUI起動(GPU)」を押すと、PC側の前面ウィンドウ（CMD等）に `D:\00_START_ComfyUI_GPU.bat` の文字列がそのまま入力される。**Enterは押されない**
- **G-c**: 「実行」ボタンを別に押したときにだけEnterが送られる
- **G-d**: `cargo test --workspace` が全pass（現在99件。既存テストの削除・弱体化なし）
- **G-e**: 凍結領域（`crates/hub-core/`・`keymaps/keymap_default.json`・`brief/`の既存設計書）への差分がゼロ
- **G-f**（段階B）: iPadから手順書1本を選ぶと、その本文がiPadに表示される。**Boardは`127.0.0.1`のまま**で、Board単体はLANから見えない
- **G-g**（段階B）: 中継の許可リストに無いIDを要求すると、Hubが拒否する（code+cause付き1行ログ・D9）

---

## 7. 着手前に必ず片付けること

**作業ツリーが汚れている。この上に新規ファイルを足すと、どこまでが今回の変更か切り分けられなくなる。**

2026-09-05時点の`git status`:

- 未コミットの変更: `DEVBOARD.md` `INDEX.md` `STATE.md` `crates/proto-hub/src/{main,state,ws}.rs` `static/deck.html`
- 未追跡: `brief/mockup/screen_mock_streamdeck_v0.8.html` `brief/proposals/P-003_streamdeck_v2.md` `static/panel.html` `prototypes/` `reports/handoff_*.md`
- `origin/main`より**3コミット先行**（`dca7fd7` `b6dab96` `3a83df3`）

STATE.mdの「未回答1〜2」（T15〜T19のコミット可否・push可否）が未解決のまま。
**段階A-2（JSONを置くだけ）は混ざっても実害が小さいが、Rustに触る作業（案1・段階B）に入る前には必ずコミット判断を取ること。**

---

## 8. 未確認・要確認事項（専属チャットが潰すこと）

1. **Ollamaがどのマシンにあるか**。この家PCでは実行ファイルが見つからなかった（§4.3）
2. **`OLLAMA_MODELS`の実際の設定値**
3. **split面（`/kb`）をiPad1枚で開いたときに左右どちらが表示されるか**。案3の可否がこれで決まる
4. **ComfyUI画面内のキーボードショートカット**（Queue Prompt等）。推測でvkを置かないこと
5. **外出先のネットワークでiPad↔PCが通るか**。公共WiFiはAP isolationで遮断されることが多い。代替手段（PCのテザリング等）の確認が要る
6. **Board側の`SSD_ROOT = Path("D:/")`ハードコード**。持ち出し先でSSDが`E:`等になるとBoardは起動しない。**これはKeyDeckではなくBoard側（別プロジェクト）の修正案件**なので、本依頼のスコープ外として別途起票すること

---

## 9. やってはいけないこと（CLAUDE.md再掲＋本件固有）

- `crates/hub-core/`を1行でも変更する
- `keymaps/keymap_default.json`を編集する
- 任意パス書込・任意コマンド実行APIを作る（不変条件6）
- クライアントから送られた文字列をそのまま実行系に渡す（不変条件1）
- フロントにフレームワーク・ビルドツール・CDN依存を入れる（D2）
- **Operations Boardを`0.0.0.0`にbindする**（認証ゼロのファイル書込APIがLANに露出する）
- **`/__shutdown`を中継の許可リストに入れる**
- 仕様が曖昧なまま解釈して進める（`brief/spec_return_log.md`へSR起票して停止すること）

---

## 10. 昇格条件5項目の自己評価（vision §2）

① MVP完成 — 済（KeyDeck本体は稼働中）
② 利用場面1つに定まる — 定まっている（外出先SSD作業でのComfyUI／Ollama操作）
③ 受け入れ基準を先に書ける — §6に記載済み
④ 既存Protocol内か — **段階Aの案2・案3は完全に既存内**。案1（`/ipad`のkeymapId可変化）と段階B（Board中継）は新規D番号の裁定が要る
⑤ 10日以内の粒度 — 段階Aのみなら小粒度。段階B込みは要分割

---

## 11. 裁定

- §1の4点（分岐しない・定型のみ・Hub中継・3区分）は**ユーザー裁定済み（2026-09-05）**
- §2.3の案1／案2／案3の選定は**未**。専属チャットが§8を潰したうえで提案し、ユーザー裁定を取ること
