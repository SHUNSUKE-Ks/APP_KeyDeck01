# APP_KeyDeck01 STATE

最終更新: 2026-07-29

## 目的（変わらない）
PC（Rust Hub）を頭脳にし、スマホ/タブレットのブラウザを「ただのボタンの板」にして、キーボード・Stream Deck等の入力デバイスに変えるアプリ。全フォーマットはコード不要のJSONで定義。完成の定義=各Surface（分割キーボード/Deck/一枚キーボード等）がJSON編集だけで自由に作り替えられ、ユーザーが実機で「使える」と判断すること。

## 実体の場所
- 作業ルート: `C:\00_master\DevApps\APP_KeyDeck01`（存在確認済み、独立gitリポジトリ、`C:\00_master`親repoとは無関係）
- リモート: https://github.com/SHUNSUKE-Ks/APP_KeyDeck01 （main。ローカルHEADと完全同期済み、fetch確認済み・未pushなし）
- 真実源: `DEVBOARD.md`（全履歴・決定事項・検証記録）／`CLAUDE.md`（不変条件・凍結領域）／`INDEX.md`（全ファイル索引）

## 現在地
- 分割キーボード・Deck・iPad一枚キーボード(Vol1.2)は実装済み、`cargo test --workspace`=54件全pass
- フォーマットのPC側自由編集（段階B: keymaps/自動発見＋`/api/reload`即時反映）まで実装・実機検証済み
- 直前に完了: 統治文書一式（CLAUDE.md/INDEX.md/vision/brain JSON/backlog/inspector草案/type ideas/app report）を作成しpush（最終コミット`1d09764`、2026-07-28）
- 次にやる予定だった: **ユーザーによるVol1.2の実機テスト最終確認 → 問題なければVol1.2を凍結（`keymap_ipad01_vol12.json`をVol名で複製+gitタグ）→ その後T9(VIAL型編集GUI)着手**

## 未回答・判断待ち【最重要】
1. `brief/mockup/screen_mock_ipad02_v0.1.html`（互い違い配列の新レイアウト候補）を採用するか未定。ユーザー判断待ち。採用ならD24を46サブ列gridへ拡張し`keymap_ipad02.json`を新規実装
2. Vol1.2実機での「記号レイヤーのtext入力」「英数⇄日本語(ALT+GRAVE chord)」の実際の動作結果が、会話上で明示的に確認されていない（ソフト側=SendInput受理は検証済みだが、人の目でのIME挙動確認は別途必要）
3. 上記2が未確認のため、Vol1.2の凍結（Git tag作成）はまだ実行していない
4. T9（VIAL型設定編集GUI）着手の可否・タイミングはユーザー確認待ち（設計はv0.4に存在、着工合意なし）
5. `brief/keydeck_type_ideas_v1.json`・`brief/inspector_schema_draft_v0.1.json`は全て提案段階。着手する型があるかはユーザー裁定待ち

## 読む順
1. `INDEX.md` — 全ファイルの地図・読み順
2. `CLAUDE.md` — 全AI必読規則（凍結領域・不変条件6箇条）
3. `DEVBOARD.md` — 決定事項ログ・タスク進捗・検証記録の全時系列
4. `brief/keydeck_brain_v1.json` — ユーザー傾向・罠PF1〜8（注: 内部のopen_threadsは本STATE.mdの方が新しい。本ファイル優先）
5. `brief/keydeck_design_v0.4.md` — 現行設計書（D1〜D26）

## 落とし穴
- Hub再起動のたびにtokenが変わる（D8）。「繋がらない」と言われたら最初にHub生死とtoken鮮度を疑う
- CSS Gridの`1fr`は内容の最小幅を下回れない。狭いWebView（QRスキャナ内蔵ブラウザ等）で表示崩れが実際に発生した（修正済み。同種の罠に注意）
- Browser paneのスクリーンショットツールがこのプロジェクトの検証中に頻繁に固着した。`get_page_text`/`javascript_tool`のDOM検証で代替すること
- 英数⇄日本語=ALT+GRAVE chordはMS-IME既定挙動への依存。他環境で効かない可能性は未検証（未回答2参照）
- 本ファイル作成時点で`proto-hub.exe`がPID起動中・port 8770 LISTENING中だった（前回セッションの起動が残っている）。作業再開時は生死を確認し、必要なら`start_hub.cmd`で再起動（tokenは再生成される）

## 履歴
- `DEVBOARD.md`（時系列の全検証記録・決定事項ログ、本体）
- `reports/report_keydeck_v0.2_verification.md`（Opus独立検証シートの書式見本）
- `brief/spec_return_log.md`（SR-001裁定済み、現在オープンなSRなし）
