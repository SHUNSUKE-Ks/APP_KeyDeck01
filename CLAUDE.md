# APP_KeyDeck01 — AI作業規則（全AI必読。ユーザー開発の根本アプリ）
> **迷ったら `INDEX.md`（全ファイル索引・読み順つき）を最初に開くこと。**

このリポジトリはPC基地局(Rust Hub)＋ブラウザ端末のキーボード/Deckアプリ。**React Native不使用・素のWeb端末＋Rust Hub**という現行方式は確定アーキテクチャであり、提案なしに変更してはならない。

## 正（source of truth）の所在
- 状況・決定事項: `DEVBOARD.md` ／ 設計書: `brief/keydeck_design_v0.4.md`（D1〜D26。旧版v0.2/v0.3も参照値）
- 見た目の正: `brief/mockup/screen_mock_v0.4.html`（実装はこのトークン・配置を踏襲）
- 配置の正: `brief/ref_ipad_keyboard_parts_v1.md`
- 仕様の曖昧さは**勝手に解釈せず** `brief/spec_return_log.md` へSR起票して停止

## 変更禁止（壊すと機能が消える）
- `crates/hub-core/` — vendored凍結。1行も変更禁止
- `brief/` 配下の設計書・モック（SR起票のみ可）
- `keymaps/keymap_default.json` — リセット用の不変原本
- Vol凍結版キーマップ（例: vol1.1系）— 複製して新Volを作ること
- Gitタグ `format-*` は復元ポイント。削除・上書き禁止

## アーキ不変条件（理由付き。違反PRは差し戻し）
1. **クライアントが送ってよいのは「位置ID」か「面が発する正規化状態」のみ**で、意味の解決は必ずHub側（D5・**D28**）。任意vk/文字列を受けるWS APIを追加しない — 許可リスト防御が無効になるため
   - **D28（2026-08-01・ユーザー裁定）**: 連続値を出す面（トラックボール等）は `surface.state` を送ってよい。ただし次を全て満たすこと。**クライアントは「何に繋がるか」を指定できない**のが核心
     - 送ってよいのは `surfaceId` ＋ 上下限つき数値のみ（`delta.dx`/`dy` は ±200 でクランプ。超過は `SURFACE_STATE_RANGE` で拒否）
     - **出口(`binding`)はHub側のJSONだけが決める**。クライアントは binding を送れない・選べない
     - `binding` は事前登録の許可リストのみ。未知の値はロード時に拒否
     - 送信レート上限あり（125Hz相当。超過分は捨てる）
   - 根拠: 一般のUSB HIDマウス／トラックボール（Logicool・Elecom等）と同一の形＝相対移動量2つであり、業界標準から外れた設計ではない。`Win+R`等が既に送れる以上、リスクの実質的増分も小さい
2. **レイヤー意味論はD3固定**: {0}∪momentary∪toggled の番号最大優先・transフォールスルー・決定的
3. **D9ログ規約**: エラーは必ず code+cause 付き1行（Hub tracing＋クライアントconsole.error）。ランタイム入力起因のpanic禁止
4. **D2**: PWA機構・フロントフレームワーク・ビルドツール・CDN依存の導入禁止（端末は素のHTML+JS）
5. **D20**: 文字直接入力はSendInputのKEYEVENTF_UNICODE（text action）。クリップボードを黙って書き換えない
6. 書き込み系APIは **`keymaps/layers/`・`layouts/`・`keymaps/keymap_<id>.json`** ＋スキーマ検証＋`.bak`バックアップ付きのみ（D22）。任意パス書込・任意コマンド実行APIは絶対に作らない
   - **`layouts/` の追加（2026-09-07・ユーザー裁定）**: 区画エディタ（P-005 段階D）が保存できるようにするため。次を全て満たすこと
     - 書き先は `layouts/layout_<layoutId>.json` **のみ**。ファイル名はクライアントから受け取らない
     - `layoutId` は `[a-z0-9_]{1,64}` のみ許可（`ws::layout_id_is_safe`）。**パスを組み立てる前の唯一の関門**であり、ここを緩めると任意パス書込になる
     - 受け取った本文はそのまま書かず、**Layoutとして解釈し直し整形したもの**を書く（余計なフィールド・書式の揺れを持ち込まない）
     - 書く前に `.bak` へ退避し、書いた後に**全体を読み直して検証**する。失敗したら巻き戻して現行構成を維持する
     - 検証は既存の `startup::load_startup_data` を再利用する（保存専用の緩い検証を作らない）
   - **`keymaps/keymap_<id>.json` の追加（2026-09-07・ユーザー裁定）**: キーの位置編集（P-007 段階B）のため。`layouts/` と同じ防御に加えて次を守ること
     - 書き先は `keymaps/keymap_<keymapId>.json` **のみ**。`keymapId` は**いま読み込まれているキーマップに実在するものだけ**を受け付ける（ファイル名をクライアントから受け取らない）
     - **書き換えてよいのは `board` だけ**。`kind` / `layerFiles` / `description` は既存ファイルから読んでそのまま残す
     - **`layerFiles` は絶対にクライアントから受け取らない**。ここはファイルパスの配列であり、受け取ると任意パス読み取りになる
     - 分割キーボード（`kind: split`）は `board` を持たないので対象外。拒否する

## 品質ゲート（マージ・完了報告の条件）
- `cargo test --workspace` 全pass（39件以上を維持。既存テストの削除・弱体化禁止）
- 既存動作の回帰確認: split左右のレイヤー同期／Deck発火／QR表示（DEVBOARD検証記録の再現手順）
- 各タスク完了時にDEVBOARD検証記録へ実行コマンドと結果を1行追記

## レビュー体制
- 大きめの変更後は守護エージェント `keydeck-guardian`（`.claude/agents/`）で規則違反と回帰を点検すること

## 技術トピックカードの登録
動作を確認できた機能は、`C:\00_CreatorHub\knowledge\TechTopicCards\` へ再利用できる形で登録する。
KeyDeck固有の判断ではなく他プロジェクトへ持ち出せる技術（データ構造・UIパターン・検証手順など）が対象。

手順:
1. `knowledge/TechTopicCards/README.md` と `schema/tech_topic_card.schema.json` を読む
2. id接頭辞は `KEYDECK_`（例 `KEYDECK_LOGIC_001`）
3. **「実際に動くと確認できたもの」だけをカードにする**。設計だけ・未検証のものは `status` を正直に書く（`working`/`partial`/`planned`）
4. `verification.files` に書くパスは実在を確認してから書く。実在しないファイル・関数・数値を書かない（最重要規則。嘘のカードはAIの参照資産として無効になる）
5. `knowledge/TechTopicCards/cards/<ID>.json` を作る
6. `node knowledge/TechTopicCards/tools/build_cards.mjs` で `INDEX.md` を再生成する（`verification.files` の実在をここで機械的に検査する）
7. 何をカードにして何を見送ったか、理由つきで報告する

前例: `KEYDECK_LOGIC_001`（位置と中身を1本の履歴で戻す — `static/keys.html` のundoStack）
