# 作業依頼: KeyDeck（APP_KeyDeck01）を CreatorHub へ移す

作成 2026-09-22 ／ 依存調査は 2026-09-22 時点の実測（grep・ショートカット・git の状態を直接確認）

---

## 0. この依頼の要点

```
移す元   C:\00_master\DevApps\APP_KeyDeck01            （独立git。origin = https://github.com/SHUNSUKE-Ks/APP_KeyDeck01.git）
移す先   C:\00_CreatorHub\projects\APP09_KeyDeck        （案。§1 の決定1）
やり方   コピー → 新しい場所で検証 → 参照を付け替え → 元の場所は消さずに「移動済み」の札を立てて残す
```

**Rust のコードは直さなくてよい。** Hub は `layouts/` `keymaps/` `decks/` `surfaces/` `apps/` `static/` を
**起動時の作業フォルダからの相対パス**で読む（`crates/proto-hub/src/main.rs`・`ws.rs` の `ServeFile::new("static/...")`）。
`start_hub.cmd` が `cd /d "%~dp0"` してから `cargo run` するので、どこに置いても動く。
Cargo の workspace も相対パスのみ（`Cargo.toml` の members は `crates/...`）。他リポジトリからの path 依存もゼロ。

**壊れるのは「外から絶対パスで指しているもの」だけ。** その一覧が §4。

---

## 1. 始める前に人が決めること

| # | 決めること | 案（推し） | 根拠 |
|---|---|---|---|
| 1 | 移動先のフォルダ名 | `C:\00_CreatorHub\projects\APP09_KeyDeck` | CH の `projects/` は `接頭辞_名前` の1階層。`APP08_CookMemo` までフォルダが存在するので次は APP09（登記簿 `C:\00_CreatorCompass\registry.json` には APP05/06 もある。重複しないことを登記前に再確認） |
| 2 | git の扱い | **独立gitのまま入れ子にし、CH の `.gitignore` に1行足す** | CH には前例が4件ある（`GAME08_VoiceOfCards`・`TOOL05_JamTracker` は GitHub あり、`TOOL01_ProjectBoard`・`ENG01_GameEngineSuite` は remote なし）。KeyDeck も GitHub remote をそのまま使える。CH の履歴に混ぜる（subtree）と、GitHub 側との関係が切れる |
| 3 | 技術スタックの扱い | CH の「別枠」として記録する | CH の標準は SolidJS（`C:\00_CreatorHub\AGENTS.md`「技術スタック」）。KeyDeck は Rust ＋ 素の HTML/JS で、しかも**素の HTML/JS は KeyDeck の不変条件 D2**（フレームワーク禁止）。SolidJS に寄せてはいけない。`TOOL02_HyperFrames` と同じ「別枠」に書き足す |
| 4 | 検証コマンドの統一 | `package.json` を置き `"verify": "cargo test --workspace"` だけ持たせる（任意） | CH は各プロジェクトの検証を `npm run verify` に揃える決まり。中身は自由。置かないなら「KeyDeck は cargo test」と README に書く |
| 5 | 元の場所の扱い | **消さない。** `MOVED.md` を置いて残す | 利用者の方針「終わったもの・置き換わったものは消さずに状態を変えて残す」 |
| 6 | 登記簿の正本はどれか | `C:\00_CreatorCompass\registry.json` | `registry.json` 自身に「登記簿の正本。旧 `C:\00_ProjectDirector\03_Mission\projects.json` は 2026-08-23 で凍結」とある。**ただし `C:\00_CreatorHub\AGENTS.md` 55行は今も旧PDの projects.json を正本と書いている**（食い違い。直すかどうかを決める） |
| 7 | 移すタイミング | トラックボールの相談（`reports\consult_20260922_trackball_drag.md`）を出したあと | 相談用プロンプトは**旧パスの絶対パス**で書いてある。先に移すと相談相手が読めなくなる。先に移す場合はそのファイルも §4 の対象に入れる |

以前の決定との関係: `external_apps.json` と PD の台帳には「今は移動しない。Ver1 で大きなバージョンアップがある時を機に CH へ移す」（2026-08-23）と記録がある。**今回はこの方針を人が覆す判断**なので、§4 の該当箇所に「いつ・なぜ移したか」を書き残すこと。

---

## 2. 前準備（移す前に元の場所でやる）

1. **Hub を止める**。Hub の黒いウィンドウを閉じる（`target\debug\proto-hub.exe` が動いていると、コピーの途中で使用中になる）
   - 確認: PowerShell で `Get-Process proto-hub` が何も返さない
2. **未コミットを片付ける**（2026-09-22 時点で27件）
   - 変更: `DEVBOARD.md`、`crates/proto-hub/src/{layout,startup,ws}.rs`、`keymaps/keymap_{mouse_left,youtube01}.json`、`keymaps/layers/{mouse_left,youtube01}_layer0.json`、`layouts/layout_ipad_youtube.json`、`start_hub.cmd`、`static/{components.js,editor.html,keys.html,layout.html,trackball.html}`、`surfaces/trackball.json`
   - 新規: `keymaps/keymap_{click_left,screenshot_region}.json` とその layer、`layouts/layout_{iphone7_portrait,iphone7_youtube,shortborad}.json`、`static/trackball_v1_1.html`、`reports/consult_20260922_trackball_drag.md`、この計画書
   - **一度も git に入っていないもの**: `00_勉強フォルダー/`（技術書の原稿一式・約1.4MB）と `.harness/`（作業報告・約84KB）。入れるか、今後も外で持つかを決める（どちらにしてもコピーでは一緒に運ばれる）
   - `mouse_left` は中身が Win+1〜3 に書き換わっている（左クリックではない）。意図した変更かを人に確認してからコミット
3. **復元点を打つ**: `git tag pre-move-20260922`（`format-*` は復元点として別の意味で予約済みなので使わない）
4. GitHub へ push するかは人が決める（共有先に出る操作のため、確認なしに push しない）

---

## 3. 移す

1. コピー（`target/` 2.8GB は運ばない。git 管理外で、作り直せる）
   ```powershell
   robocopy "C:\00_master\DevApps\APP_KeyDeck01" "C:\00_CreatorHub\projects\APP09_KeyDeck" /E /XD target /R:1 /W:1
   ```
   - `.git`・`00_勉強フォルダー`・`.harness`・`.claude\agents` も一緒に運ばれることを確認する
   - `data/`・`*.bak`・`dist/` も .gitignore 対象だがコピーはされる（害は無い）
2. 新しい場所で git が生きていることを確認: `git -C "C:\00_CreatorHub\projects\APP09_KeyDeck" status` と `git remote -v`
3. CH 側の git から外す: `C:\00_CreatorHub\.gitignore` に `projects/APP09_KeyDeck/` を1行足す（既存の4行と同じ書き方）

---

## 4. 参照の付け替え（ここが本題）

### 4.1 直さないと動かなくなるもの

| 場所 | 何を直すか |
|---|---|
| スタートメニュー `C:\Users\enjoy\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\KeyDeck Hub.lnk` | 実行先を新しい `start_hub.cmd`、作業フォルダを新しい場所、アイコンを新しい `static\icons\keydeck.ico` へ（2026-09-22 に `start_hub.cmd` 経由へ変えたばかり。変更前は `%TEMP%\KeyDeck Hub.lnk.bak`） |
| `C:\00_CreatorHub\.claude\launch.json` | `keydeck-hub`（`cd /d C:\00_master\DevApps\APP_KeyDeck01 && cargo run -p proto-hub`）、`keydeck-mock`（`...\brief\mockup\serve.mjs`）、`keydeck-study`（`...\00_勉強フォルダー`）の3件 |
| `C:\00_master\.claude\launch.json` | 29行 `"cwd": "C:/00_master/DevApps/APP_KeyDeck01"`（00_master 側で起動する設定。残すなら新パスへ、使わないなら人に確認して外す） |
| `C:\Users\enjoy\.codex\skills\bookmaker\SKILL.md` | 17〜20行・36〜37行（CLAUDE.md / INDEX.md / DEVBOARD.md / 勉強フォルダーの schema と example）。**Codex の技術書執筆がここを読みに行く** |
| `（移動後）00_勉強フォルダー\14_ClaudeCode引継ぎ_絶対パス参照一覧.md` | ほぼ全行が旧パス。同じ書き方の姉妹ファイル（`00_はじめに.md`・`02_...初稿.md`・`06_...第2稿...md`・`12_ClaudeCode引継ぎ_書籍執筆依頼.md`・`15_引継ぎ_次のチャットへ_20260915.md`・`32_キーボード編集View_デザインメモ.md`・`examples\fn-momentary-bookmaker-v1.1.json`）も `grep "00_master\\DevApps\\APP_KeyDeck01"` で洗って直す |

### 4.2 登記・目録（古い場所を指したまま残る。動作は止まらないが、AIが嘘の場所を案内する）

| 場所 | 何をするか |
|---|---|
| `C:\00_CreatorCompass\registry.json` | APP09（決定1の名前）で1件足す。`path` と `reportRoot` は新しい場所、`status: "active"`、`pathVerified` は移した日。`note` に「2026-09-xx に C:\00_master\DevApps\APP_KeyDeck01 から移動。独立git（GitHub）を入れ子で保持」 |
| `C:\00_CreatorHub\knowledge\ExternalApps\external_apps.json` と同 `README.md` | KeyDeck は「CH の外にあるアプリ」ではなくなる。`keydeck` の項目を消すのではなく、`path` を新しい場所に・`state` に「CHへ移動済み（APP09）」と書く（README の「移動の基準」節にも、この件で方針を覆した旨を1行）。`keydeck_expo`（`C:\00_master\CreatorHub\DEV\KeyDeck`）と `keydeck_mockup`（`C:\00_master\MockUp\APP_KeyDeck01`）は移さないので触らない |
| `C:\00_CreatorCompass\KeyDeck\できること.md` | 6行「現物」と 216行（KeyDeck 側 CLAUDE.md のパス） |
| `C:\00_CreatorCompass\KeyDeck\記入例.json`・`依頼\REQ-20260912-001.json`・`依頼\REQ-20260912-002.json` | 中の旧パスを確認して直す（依頼書は他プロジェクトがKeyDeckへ出す依頼の窓口。未着手の依頼が新しい場所を指すように） |
| `C:\00_CreatorHub\projects\ENG02_UnityLab\README.md` | 91行「`C:\00_master\DevApps\APP_KeyDeck01` を直接編集 ✗」の例示パス |
| `C:\00_CreatorHub\knowledge\TechTopicCards\目録.md` | 39行・67行の `KEYDECK` を「未登記」から APP09 へ |
| `C:\00_CreatorHub\knowledge\TechTopicCards\cards\KEYDECK_LOGIC_001.json` | `source.root`。**commit を固定した記録**なので、直すかどうかは人が決める（直さなくても壊れない） |
| `C:\00_CreatorHub\knowledge\BookStandard\実録_KeyDeck技術書_本の作り方.md` | 中の旧パス（技術書づくりの実録。読み物として参照されるなら直す） |
| KeyDeck 自身の `README.md`・`STATE.md`・`INDEX.md`・`CLAUDE.md`・`.claude\agents\keydeck-guardian.md` | 旧絶対パスの記述（起動手順・外部登録の説明）。`reports/`・`.harness/reports/`・`DEVBOARD.md` の過去の記録は**その時点の事実なので直さない**。`DEVBOARD.md` には移動の記録を1件足す |

### 4.3 付け替えたあとに作り直すもの（生成物）

| コマンド | 何のため |
|---|---|
| `node C:\00_CreatorHub\projects\TOOL01_ProjectBoard\tools\scan_projects.mjs` | ProjectBoard の `src/data/generatedProjects.ts`。登記簿で CH 配下にあるものを「プロジェクト」として、`external_apps.json` を「外部アプリ」として拾う |
| `node C:\00_CreatorHub\projects\TOOL01_ProjectBoard\tools\scan_reports.mjs` | `generatedReports.ts`。**00_master は走査対象外**なので、移すと KeyDeck の報告書が初めて ProjectBoard に載る |
| `node C:\00_CreatorHub\tools\build_index.mjs` | `C:\00_CreatorHub\INDEX.md`（手で書かない生成物）。登記しないと「未登録」と出る |

### 4.4 触らないもの（理由つき）

| 場所 | 理由 |
|---|---|
| `C:\00_ProjectDirector\` 配下（`台帳\アプリ目録.md`・`台帳\ToolList.md`・Mission・受付口など約20件） | `registry.json` で PD は `status: archive`「触らない・消さない。参照のみ」。過去の記録として残す |
| `C:\00_CreatorCompass\CCへ_*.md`・`引継ぎ_Unity開発_20260912.md` | 日付入りの過去の引継ぎ文書 |
| `C:\00_master\MockUp\APP_KeyDeck01\`・`C:\00_master\DevApps\APP_ControlDeck\DEVBOARD.md`・`C:\00_master\_migration_staging\`・`C:\00_master\CHATGPTNOTELOG_0726\` | 別アプリ・過去の棚卸し・会話ログ。移動の対象外 |
| ポータブル版 `D:\00_WorkSpace\APP_KeyDeck01_portable\` と `D:\00_START_KeyDeck.bat` | 独立したコピー。`D:\00_START_KeyDeck.bat` は `%~dp0` からの相対で探すので、元の場所とは無関係 |
| `C:\00_master\DevApps\HarnessBoard\orchestra\report_roots.json` | KeyDeck は元々登録されていない |
| Claude Code の記憶 `C:\Users\enjoy\.claude\projects\c--00-master-DevApps-APP-KeyDeck01\memory\` | 空（失うものが無い）。移動後に新しい場所で開くと別の記憶フォルダになる |

---

## 5. 新しい場所で検証する

1. `cd C:\00_CreatorHub\projects\APP09_KeyDeck` → `cargo test --workspace`（2026-09-22 時点で 85＋42＋8＋7 件がすべて通る。1回目は `target/` が無いので時間がかかる）
2. `start_hub.cmd` をダブルクリック → コンソールに `startup data loaded successfully` と接続URLが出る
   - `start_hub.cmd` は UTF-8＋`chcp 65001`（2026-09-22 修正済み）。日本語が化けたら `chcp` の行が消えていないか見る
3. PC のブラウザで `/?token=…`（レイアウト編集）、iPhone で QR を読み直して `iphone7_portrait` を開き、キーとトラックボールが効くこと
4. スタートメニューの「KeyDeck Hub」からも起動できること
5. CH 側: `node tools/build_index.mjs` で APP09 が「未登録」になっていないこと

---

## 6. 元の場所を閉じる

1. `C:\00_master\DevApps\APP_KeyDeck01\MOVED.md` を置く（移した日・移した先・理由・復元点のタグ名）
2. 元の場所は**消さない・中身も直さない**。新しい場所で1週間ほど問題が無ければ、`target/`（2.8GB）だけは消してよいか人に聞く
3. 元の場所から誤って Hub を起動しないよう、`MOVED.md` の1行目に新しい `start_hub.cmd` のパスを書く

---

## 7. 報告

- CH の決まりに合わせ、作業記録は `C:\00_CreatorHub\projects\APP09_KeyDeck\AI_Memory\作業ログ\` に置く（`C:\00_CreatorHub\AGENTS.md`「背景タスクの記録」）
- 報告に含めるもの: §1 の決定内容、§4 で直したファイルの一覧（直した／直さなかった＋理由）、§5 の検証結果（件数・実機で見たもの・見ていないもの）
