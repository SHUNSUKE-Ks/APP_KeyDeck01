# APP_KeyDeck01 — AI作業規則（全AI必読。ユーザー開発の根本アプリ）

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
1. **クライアントは位置ID(keyId/slotId)のみ送信**し、キー解決はHub側（D5）。任意vk/文字列を受けるWS APIを追加しない — 許可リスト防御が無効になるため
2. **レイヤー意味論はD3固定**: {0}∪momentary∪toggled の番号最大優先・transフォールスルー・決定的
3. **D9ログ規約**: エラーは必ず code+cause 付き1行（Hub tracing＋クライアントconsole.error）。ランタイム入力起因のpanic禁止
4. **D2**: PWA機構・フロントフレームワーク・ビルドツール・CDN依存の導入禁止（端末は素のHTML+JS）
5. **D20**: 文字直接入力はSendInputのKEYEVENTF_UNICODE（text action）。クリップボードを黙って書き換えない
6. 書き込み系APIは `keymaps/layers/` 配下＋スキーマ検証＋`.bak`バックアップ付きのみ（D22）。任意パス書込・任意コマンド実行APIは絶対に作らない

## 品質ゲート（マージ・完了報告の条件）
- `cargo test --workspace` 全pass（39件以上を維持。既存テストの削除・弱体化禁止）
- 既存動作の回帰確認: split左右のレイヤー同期／Deck発火／QR表示（DEVBOARD検証記録の再現手順）
- 各タスク完了時にDEVBOARD検証記録へ実行コマンドと結果を1行追記

## レビュー体制
- 大きめの変更後は守護エージェント `keydeck-guardian`（`.claude/agents/`）で規則違反と回帰を点検すること
