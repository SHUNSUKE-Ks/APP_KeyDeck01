# APP_KeyDeck01 v0.2 検証シート（Opus独立検証・T6）

実施: Opus（検証者） ／ 2026-07-19 ／ 対象: Sonnet実装のT1〜T5
方針: 実装者の自己申告（DEVBOARD記載）を信じず、Opusが独立にビルド・テスト・起動・E2Eを再実行した結果を記す。所見はF#。

## 総合判定

**V1〜V6すべて合格。G1〜G7の受け入れ基準を満たす。** 未解決の欠陥（ブロッカー）なし。
残る人手確認は2点のみ（G1の物理的なグリフ描画・G3の物理的なミュート）で、いずれも設計が「手動E2E」と定義する部分。ソフト〜OS境界（SendInput受理）まではすべて機械検証済み。

## V項目の独立再実行結果

| V# | 内容 | 判定 | 独立再実行の証拠 |
|---|---|---|---|
| V1 | `cargo build`＋`cargo test`（12+件） | **合格** | Opusが`cargo build --workspace`クリーン成功／`cargo test --workspace`= **39 passed, 0 failed**（hub-core 7・proto-adapter-win 7・proto-hub 5・proto-keymap 20）、warning/error無し |
| V2 | G1手動E2E（2窓→メモ帳"hello"） | **合格（機械検証部分）** | "hello"=R21,L14,R24,R24,R14（左右にまたがる）をWSで注入→**errorフレーム0・`ADAPTER_SENDINPUT_FAIL`0**。Windowsビルドは実SendInputがコンパイルされ、失敗時は必ずエラー化されるため、エラー0＝OSが全キーの合成入力を受理。**物理的なグリフ描画は人手確認**（後述F4） |
| V3 | G2レイヤー同期（MO/TG・500ms以内）＋G6切替/reset | **合格** | 2クライアントで実測: MO(1) down→**第2クライアントへ4ms**で`layer.state`到達（<500ms）。MO up復帰、TG(2)トグル配信、`momentary=[] toggled=[2]`同時成立で**MO/TG独立**（D3準拠）を確認。deck.press S09(reset)→両クライアントが`activeKeymapId=default`のsurface.config受信（G6） |
| V4 | G3 Deck実発火＋export JSONスキーマ妥当 | **合格** | `/api/deck/export`をpython jsonschemaで`deck.schema.json`（keymapアクションへの外部$ref解決込み）検証→**妥当**。`Content-Type: application/json`＋`Content-Disposition: attachment`確認。deck.press S01(消音)発火エラー0 |
| V5 | G7エラー誘発4種→code＋cause付き1行 | **合格** | ①不正JSON→`WS_PARSE`（cause＋raw文脈）②未知keyId→`KEY_UNKNOWN_ID`③token不一致→HTTP 401＋Hub `WS_TOKEN_INVALID`ログ ④未知vk→**起動時拒否 exit 1**＋`LOAD_VK_UNKNOWN`（ファイル/レイヤー/keyId/不正vk名までcause明記）。①②はブラウザ＋Hub両方、③④は後述F1 |
| V6 | 本フォルダ外の無変更 | **合格** | `git status`で`APP_KeyDeck01`外の変更はすべてセッション開始時点の既存差分（kanban-note01等）で本作業由来ゼロ。D10不変条件（writing01==default）復元済み。検証プローブは全てscratchpad/tmpに隔離しプロジェクト内に残渣なし |

## 受け入れ基準（G1〜G7）の判定

- **G1**（タイプ実証）: 合格（機械検証）＋物理描画は人手（F4）
- **G2**（レイヤー同期500ms以内）: 合格（実測4ms）
- **G3**（Deck実発火）: 合格（発火・SendInput受理）＋物理ミュートは人手（F4）
- **G4**（データ駆動）: 合格。起動時に`keymaps/*.json`を実ロードすることをV5④のバイナリ再起動で確認（Sonnet一次検証でラベル編集→表示反映も確認済み）
- **G5**（回帰ガード・決定性）: 合格。proto-keymap 20テスト、`t1_4_same_input_sequence_is_deterministic`で決定性を保証
- **G6**（State切替/reset・両kb反映）: 合格（V3で両クライアント反映を実測）
- **G7**（診断可能性）: 合格（V5の4種、各1行code＋cause）

## 所見（F#。すべて軽微・非ブロッカー）

### F1【軽微・仕様文言】起動時クラスのエラーはHubコンソールのみ（V5/G7の文言）
- 設計V5は「Hub/ブラウザ両コンソールに出る」とするが、`LOAD_VK_UNKNOWN`（未知vk）はD4により**起動時拒否**であり、ブラウザは1つも接続していないためHubコンソールにしか出ない。挙動はD4通りで正しい。V5の文言が起動時クラスのエラーには当てはまらない点を明確化すると誤解が減る。修正不要（文言メモのみ）。

### F2【軽微・堅牢化】active_keymap取得の`.expect`がロック保持中でpanic経路
- 場所: `crates/proto-hub/src/ws.rs:151` `handle_key_press`内、Mutex保持中に`.expect("active_keymap_id always refers to a loaded keymap")`
- 不変条件（active_keymap_idはcontains_key検証後のみ設定）により**現状は発火しない**。ただし万一発火するとMutex毒化→以降全ハンドラの`.lock().unwrap()`が連鎖panicし、D9「panic禁止」の精神に反する。`emit_error(INTERNAL)`で優雅に扱う形が望ましい。今回のスコープでは非ブロッカー。

### F3【軽微・効率】キー押下ごとにキーマップ全体をclone
- 場所: `crates/proto-hub/src/ws.rs:152`。プロトタイプ規模では無害。将来キーマップが大きくなる場合は`Arc<Keymap>`化を検討。

### F4【観察・人手確認の残り】物理的な描画/ミュートのみ人手
- V2のグリフがメモ帳に出る／V3のミュートが実際にトグルする、の2点は焦点を持つウィンドウ依存で、設計も「手動E2E」と規定。ソフト〜SendInput受理までは機械検証済み。**ユーザーが実機（またはメモ帳フォーカス）で一度だけ確認**すれば全ゴール達成。手順は`cargo run -p proto-adapter-win --example smoke_notepad`（メモ帳）またはREADME記載のAndroid接続手順。

## コード品質の所見（良かった点）

- レイヤー解決は`{0}∪momentary∪toggled`の番号最大優先＋transフォールスルーを正しく実装、決定的。20テストが境界（trans飛ばし・番号最大優先・up無処理）まで押さえている。
- 許可リストはロード済みJSONに現れるKey/Chordのみ（D5）。クライアントは位置IDのみ送信で、任意vk送信は構造的に不可能。hub-coreの`CommandRegistry`を実際に再利用（新規発明ゼロ）。
- Deckローダはmo/tg/transをロード時に拒否（キーボード専用アクションのDeck混入を防ぐ）、slotId重複も検出。
- 層契約遵守: proto-keymapはstdのみ（serdeは可）でThree.js級の外部依存なし、serde/WSはhub-server層に閉じている。
- adapterは自動テストで実キーを飛ばさず（CI安全）、実SendInputは手動example分離という妥当な設計判断。
