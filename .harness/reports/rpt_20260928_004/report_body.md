# REQ-20260928-001 への返事 — Note Story 用の盤面

```
返答日    2026-09-28
担当      KeyDeck 統括チャット（Claude Code / Opus 5.5）
状態      作成済み。実機（iPad）と実際の Note Story では未確認
```

## 作った盤面

**board id: `note_story`**（iPad 横・12×9）

端末で開く: `http://<PCのIP>:8770/layout?id=note_story&token=…`（QRギャラリー `/connect` にも出る）
PCで編集: ナビの「執筆部門（Note Story）」→ 盤面 iPad（`/?id=note_story`）

```
┌───────────────── 話者 note_cast（8×4） ─────────────────┬ 移動 ┬ ホイール ┐
│ 人物1  人物2  人物3  人物4  人物5                          │ ↑前  │          │
│ 人物6  人物7  人物8  人物9  ナレーション                   │ 続ける│  上下に  │
│  （ALT+1〜9 / ALT+0）                                      │ ↓次  │  こする  │
├──────── 書く道具 note_tools（6×5）──┬ 定型文 note_palette ┼── ふち ──┤
│ [Char]F13  [Scene]F14  [演出]F15     │ 柱：／備考：／       │ ?    戻る │
│ 柱：       備考：      2カラムF16    │ ナレーション：「」／ │ Esc キー確認│
│                                      │ [bg_town_night]／…  │ 開く 既定へ│
└──────────────────────────────────────┴─────────────────────┴──────────┘
```

| 部品 | id | 中身 |
|---|---|---|
| 話者 | deck `note_cast`（5×2） | C1〜C9 = ALT+1〜9、C0 = ALT+0（ナレーション） |
| 行の移動 | keymap `note_move` | ALT+UP ／ CTRL+ENTER ／ ALT+DOWN |
| スクロール | keymap `wheel_scroll`（既存） | mouse.wheel。**マウスカーソルの下の窓**が動く |
| 書く道具 | keymap `note_tools` | F13 ／ F14 ／ F15 ／ text「柱：」／ text「備考：」／ F16 |
| 定型文 | deck `note_palette`（list・10件） | 柱：／備考：／ナレーション：「」／[bg_town_night]／[bg_room]／# ／// ／{}／[]／()（すべて text） |
| ふち | keymap `note_edge` | CTRL+SLASH ／ CTRL+Z ／ ESC ／ F17 ／ Note Story を開く ／ 既定の盤面へ |

## 人物の名前とアイコンを入れる手順

盤面はそのままで、**Deck の中身だけを差し替える**作りにした。

1. Note Story の「KeyDeckへ書き出し」で、**Deck名を `note_cast` にする**（既定の `chara_…` から書き換える）
2. PC の KeyDeck で Deck編集 `/deckedit` →「取り込み」→ 書き出したファイルを選ぶ
3. 「既にあります。上書きしますか？」→ OK（前の中身は `.bak` に残る）

`note_cast` の並び（C1〜C9 のあと C0）と格子（5×2）は、`keydeckExport.ts` の書き出しと同じにしてある。
人物が9人より少ない作品では格子が小さくなるが、盤面の区画は変えなくてよい。

## Note Story の起動

ふちの「Note Story 開く」は `apps/apps.json` の `note_story`
（`C:\Program Files\Google\Chrome\Application\chrome.exe --app=https://kanban-note01.vercel.app`）を起動する。
**B-002 が直るまで、起動と盤面の移動は別のボタン**にしてある（依頼の制約どおり）。

## 確認したこと

- `cargo test --workspace` = 158 passed。実データのテストに、この盤面と5部品の存在、ALT+0〜9・ALT+UP/DOWN・CTRL+ENTER・CTRL+SLASH・F13〜F17 が許可リストに載ることを追加した
- 本物の Hub を再起動して読み込み成功（keymaps 22 / decks 6 / layouts 10 / apps 2）
- レイアウト編集で「✓ 重なり・はみ出しなし」
- 端末画面を iPad 横の実寸（1366×1024）で PC のブラウザに描いて目視。**押してはいない**（本物の Hub なので PC にキーが飛ぶため）

## まだのこと（実機では1つも確認していない）

- 受け入れ基準7項目の実機での通し（iPad → Chrome アプリ窓の Note Story）
- 取り込み（2026-09-25 分）を実機の Hub で通すこと。上の手順がそのまま確認になる
- F13〜F17 は Note Story の次のデプロイまで、Note Story の 設定 → ショートカット → PC タブで「押して登録」が要る

## 提案・返事

- **話者ラジアルは入れなかった。** 人物Deckは名前とアイコンが見えるが、ラジアルは開くまで誰が何番か見えず、取り込みで人物が入れ替わっても名前が追従しない。両方置くと同じ ALT+n が2か所になるだけなので、まず Deck 1本で使ってみてほしい。片手で目を離して押したい場面が出たら、そのとき足す
- **Note Story 側に足してほしいキー: いまのところ無い。** 依頼の操作はすべて今の辞書で送れた
- **Phase ごとの盤面の分け方（notes への返事）**: まず「会話ログ」と「執筆デスク」の2枚に分けるのがよさそう。いまの1枚は両方を兼ねているが、執筆デスクには REQ-002 のあとテンキーの 3×3 が入るので、そこで分けるのが自然。ギャラリーは ← → と Esc だけなので、盤面を作るより既存の十字キー（`dpad_arrows`）で足りる
- **テンキーの 3×3 の並び（REQ-002 の質問）**: 盤面は**テンキーと同じ 7 8 9 / 4 5 6 / 1 2 3** を勧める。キーボードのテンキーと同じ手の感覚で押せるほうが、画面を見ずに押せるため。画面の 9グリッドを合わせるかは Note Story 側の判断で

REQ-20260928-002（テンキー・B-002）は、Rust の本体を変えるので別の作業として進める。
