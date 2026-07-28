# アプリ作成報告書 — APP_KeyDeck01

meta:
  date: 2026-07-20
  owner: peakexperience
  status: active（実機テスト待ち）
  category: 開発根本アプリ / 入力デバイス基盤
  purpose: 他のアプリ・レジストリから参照する時の一枚サマリ

## 概要

| 項目 | 内容 |
|---|---|
| 名前 | APP_KeyDeck01（KeyDeck） |
| 一言で | PCをHub（頭脳）にして、スマホ/タブレットのブラウザをキーボード・Stream Deckにする基盤 |
| 位置づけ | ユーザーの**開発の根本アプリ**。今後、対象アプリごとに最適な入力デバイスを後付けしていく土台 |
| リポジトリ | https://github.com/SHUNSUKE-Ks/APP_KeyDeck01（独立repo。`C:\00_master`親repoとは無関係） |
| ローカルパス | `C:\00_master\DevApps\APP_KeyDeck01` |
| 技術スタック | Rust（axum/tokio、Hub側）＋ 素のHTML/JS（端末側。フレームワーク不使用） |
| 起動 | `start_hub.cmd`ダブルクリック → PCで `http://<LAN-IP>:8770/` → QRを端末で読む |
| 索引 | `INDEX.md`（このリポジトリ内の全ファイル地図。他AIはここから読む） |

## できること（実装済み・テスト54件で保証）

- 分割キーボード2台（Android等）が**Hub経由で常時同期**（片方のレイヤー切替がもう片方の画面にも即反映）
- Stream Deck面（ボタン式マクロ発火）
- **iPad一枚キーボード（Vol1.2）**: 記号レイヤー切替、IME状態に依存しない直接文字注入（`text` action）で全角/半角記号を確実入力
- QRコードで接続（URL/token手入力不要）
- **フォーマットはJSON編集だけで変更可能**（コード不要）。設定画面の「再読込」ボタンでHub再起動なしに端末へ即反映

## 現在地

- 実装フェーズ: フォーマットPC編集の段階B完了。Vol1.2実機テスト待ち→問題なければVol凍結
- 次フェーズ: T9（VIAL型編集GUI）
- 直近の課題: なし（ブロッカー無し）。ドキュメントは[`DEVBOARD.md`](../DEVBOARD.md)に全履歴

## 統治・拡張性

- `CLAUDE.md`: 全AI共通規則（凍結領域・不変条件・品質ゲート）
- `keydeck-guardian` agent: 変更後の独立点検
- `keydeck-explorer` agent: 拡張提案（`brief/proposals/`）
- 型・Surfaceの拡張アイディアは `brief/keydeck_type_ideas_v1.json`、Inspector面は `brief/inspector_schema_draft_v0.1.json` に設計済み（未着工）

## 他アプリ/レジストリからの参照時の使い方

- 「今このアプリで何ができるか」を知りたい → 本ファイルの「できること」
- 「どう触ればいいか」を知りたい → `README.md`
- 「設計判断の理由」を知りたい → `brief/keydeck_design_v0.4.md` のD番号
- 「次に何をすべきか」を知りたい → `DEVBOARD.md` 末尾の未完タスク
