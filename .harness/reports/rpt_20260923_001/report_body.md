# レイアウト編集の新View（/editor_v2）と十字キーの矢印キー化

## 完成状態

- メニュー PC部門に「レイアウト編集 V2（新View）」が増え、`/editor_v2` で開ける。旧「レイアウト編集」（`/`）はそのまま残っている。
- 十字キー `dpad01` は矢印キー（UP/LEFT/RIGHT/DOWN）を押し続ける（key.hold）。

## 変更内容

- `static/editor_v2.html`: editor.html の写しから「置ける部品」だけ作り替え。
  - 種類チップ（件数つき）＋「未配置のみ」、種類別の見出し、未配置を先に並べる
  - カードに中身の実物ミニプレビュー（components.js で描いて縮小）と中身の文字（キーのラベル／Deckのボタン名）
  - 検索が中身の文字にも当たり、当たった所に印
  - 乗ると詳細（その部品が要る縦横比の実物・説明文全文・最小サイズ）
  - 格子へドラッグで落とした位置に置く（重なる場合は置かずに知らせる）。クリックで空きへ置く従来動作も残す
- `static/components.js`: NAV_TREE に1行。`crates/proto-hub/src/ws.rs`: `/editor_v2` の ServeFile を1行。保存は既存 `/api/layout/save` のままで、書き込み口は増えていない。
- `keymaps/layers/dpad01_layer0.json` ほか: vk を W/A/S/D → UP/LEFT/RIGHT/DOWN（Sonnet）。他の説明文の「dpad01（WASD）」も合わせて直した。

## 検証結果

- `cargo test --workspace` 152 passed。
- 模擬 config を差し込んだ写しで画面を確認: 部品22件、「esc」で youtube01 のみ、リスト4件、未配置19件、重なりドラッグは拒否、空きへのドラッグで (2,10) 3×3 に配置。
- 未実施: 実Hubでの表示・保存（稼働中Hubが旧ビルドのため、再起動が要る）。

## 成果物

- static/editor_v2.html / static/components.js / crates/proto-hub/src/ws.rs / keymaps/layers/dpad01_layer0.json / keymaps/keymap_dpad01.json / DEVBOARD.md

## 次の行動

- Hub を再起動して `/editor_v2` を実機で確認し、保存まで通す。
- 十字キーの矢印キー出力を実機で確認。
- V2 を標準の「レイアウト編集」に昇格するかを判断する。
