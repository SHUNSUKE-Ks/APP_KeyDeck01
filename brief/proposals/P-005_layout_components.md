# P-005 レイアウトと部品（section / Component / slot の三層化）

> **改番の記録**: 当初P-004として起票したが、ほぼ同時刻に別セッションが
> `P-004_ssd_field_deck.md` を同じ番号で起票していたため、こちらをP-005へ改番した
> （向こうは専属チャットへの依頼文で既にP-004を参照していたため、被害の小さい側を動かした）。
> コード内コメントの `P-005 段階A` 等も同時に付け替え済み。

起票: 2026-09-05 ／ 起票者: Claude Code（Opus 5。ユーザーの設計質問への回答として）／ 状態: **段階A/B/C 完了（2026-09-05）／段階D（配置GUI）のみ未着手**

ユーザー要望:

1. iPad面の空きスペースに新しい部品を置きたい
2. 右の空き = **ゲームの十字キー**（十字方向に動かせる）
3. 左の空き = **コピペリスト**（押すとペーストされる。label:ファイルパスの組）
4. Stream Deck は**縦2段**でよい
5. あとで**スロットの数をHubの設定でユーザーが変えられる**ようにしたい
6. PCのStream Deckのように、**Hubで部品の配置をパズルのように変えたい**
7. 「今どんな設計になってる？ section と Component、スロットという概念で大丈夫？」

---

## 1. 今どうなっているか（事実）

### 1.1 面 = 1URL = 1つの手書きHTML。配置はHTMLの中にある

| URL | ファイル | 中身 |
|---|---|---|
| `/kb` | `static/kb.html` | 分割キーボード |
| `/deck` | `static/deck.html` | Deck |
| `/ipad` | `static/ipad.html` | 一枚キーボード |
| `/trackball` | `static/trackball.html` | トラックボール |
| `/panel` | `static/panel.html` | **Deck＋キーボード（初の複合面）** |

`/panel` は「上=Deck・下=キーボード」を**HTMLとCSSに直書き**している。
つまり**「画面は部品の組み合わせでできている」という概念はまだコードにもJSONにも無い**。
複合面を1枚作るたびにHTMLを1枚書き足す、という状態。

### 1.2 データは3つの無関係な家族に分かれている

| 家族 | ファイル | WSメッセージ | 形 |
|---|---|---|---|
| keymap | `keymaps/keymap_*.json` ＋ `layers/*.json` | `key.press{keyId, edge}` | 13列グリッドの`board` ＋ レイヤー |
| deck | `decks/deck_*.json` | `deck.press{slotId}` | `grid{cols,rows}` ＋ `pages[].slots[]` |
| surface | `surfaces/*.json` | `surface.state` / `surface.gesture` | `id`/`type`/`binding`/`gestures` |

3家族はスキーマもローダーもWSメッセージも別々で、**互いに何の関係も無い**。

### 1.3 ここまでで分かった正直な欠陥

- **`deck.grid.rows` は宣言されているだけで、一度も使われていない。**
  `schemas/deck.schema.json` は必須項目にしていて `proto-hub::deck::Grid` にもフィールドがあるが、
  描画（`deck.html` / `panel.html`）は `cols` しか見ておらず、行数は「スロットの数 ÷ cols」で
  暗黙に決まる。検証も無い。要望4「縦2段」を `rows: 2` と書いても**今は何も起きない**
- **押しっぱなしができない。** `send_key()` は `press()` の直後に `release()` を呼ぶ実装で、
  `resolve()` も `Edge::Down` でしか発火しない（`Edge::Up` は `Ignored`）。
  要望2の十字キーで「押している間ずっと動く」をやるには、ここに手当てが要る

---

## 2. 「section / Component / slot で大丈夫か」への回答

**大丈夫です。語の衝突もありません。** 対応はこうなります。

| 語 | 意味 | 今の状態 |
|---|---|---|
| **section**（区画） | 画面上の1区画。**中に部品をちょうど1つ持つ** | **無い。新設が要る**（`/panel` の上下2分割がHTMLに直書きされているだけ） |
| **Component**（部品） | 部品の種類。keyboard / deck / dpad / list / trackball | 実体は3種あるが「種類」として抽象化されていない。面ごとにHTMLが直書き |
| **slot**（スロット） | **Deck部品の中のボタン1つ**（`slotId: "S01"`） | **既にこの意味で存在**（`deck.press{slotId}`・`deck.schema.json`） |

つまり `slot` は既に「Deckのボタン」で埋まっている語ですが、ユーザーの使い方
（要望5「スロットの数を変えられるように」）も同じ意味なので**衝突しません**。
`section` と `Component` の2語を足すだけで三層が揃います。

**決め切っておきたい一点**: **1 section = 1 Component**（入れ子にしない）。
こう決めると「パズルのように動かす」が「sectionの `row`/`col` を書き換える」だけになり、
GUIも実装も一気に単純になります。

---

## 3. 提案する形

### 3.1 新設: `layouts/layout_*.json`

**既存のD24グリッド（`board`）と同じ形を1段上に持ち上げるだけ**にする。
`row`/`col`/`colSpan`/`rowSpan` はそのままCSS Gridへ転記できる、という性質も同じ。

```json
{
  "layoutId": "ipad_main",
  "description": "iPad一枚。上段に3部品、下段にキーボード",
  "grid": { "cols": 12, "rows": 8 },
  "sections": [
    { "id": "SEC-list", "row": 1, "col": 1,  "colSpan": 4, "rowSpan": 4,
      "component": { "kind": "deck",     "ref": "story_paths", "render": "list" } },
    { "id": "SEC-deck", "row": 1, "col": 5,  "colSpan": 4, "rowSpan": 4,
      "component": { "kind": "deck",     "ref": "default",     "render": "grid", "cols": 4, "rows": 2 } },
    { "id": "SEC-dpad", "row": 1, "col": 9,  "colSpan": 4, "rowSpan": 4,
      "component": { "kind": "keyboard", "ref": "dpad01" } },
    { "id": "SEC-kb",   "row": 5, "col": 1,  "colSpan": 12, "rowSpan": 4,
      "component": { "kind": "keyboard", "ref": "ipad01_vol12" } }
  ]
}
```

- 面は `/layout?id=ipad_main&token=…` の**1枚のHTML**が全レイアウトを描く。
  面を増やすたびにHTMLを書き足す今のやり方をやめられる
- **WSは1本のまま。プロトコル追加ゼロ。** `surface.config` に `layout` を1つ足すだけで、
  中身（keymap / deck）は既に配信されている

### 3.2 要望ごとの実現方法（何が新規で、何が既存で足りるか）

| 要望 | 実現方法 | 新規実装 |
|---|---|---|
| 3. コピペリスト | **既存の `text` アクションで足りる**。D20のKEYEVENTF_UNICODEでパスをそのまま打ち込む。D20は「クリップボードを黙って書き換えない」と定めているので、**クリップボード経由ではなく直接入力が正しい** | **アクションはゼロ**。描画（縦リスト）だけ |
| 4. Deck縦2段 | `decks/deck_*.json` の `grid` を書き換えて `/api/reload` | **`grid.rows` を実際に使う**ようにする（§1.3の欠陥修正） |
| 5. スロット数の変更 | 今も「JSONを書き換えて`/api/reload`」でできる（v0.5設計書の段階A/B。実装済み） | GUI（段階C）が未実装 |
| 2. 十字キー | **keyboard部品の一種**。3×3の小さな`board`に4キー置くだけ。keyIdを送るだけなので新プロトコル不要 | **`key.hold` アクション1つ**（§3.3） |
| 6. パズル配置 | `layouts/*.json` ＋ `/settings` のドラッグ編集GUI | レイアウト形式＋GUI＋**不変条件6の拡張**（§4） |

### 3.3 唯一の新アクション: `key.hold`

十字キーで「押している間ずっと動く」を実現するには、押しっぱなしが要る。
T15で `mouse.button`（`{button, down}`）を作ったのと同じ形をキーボードにも用意する。

```json
{ "id": "D02", "label": "↑", "action": { "t": "key.hold", "vk": "W" } }
```

- `resolve()` は `Edge::Down` で `key.hold{vk, down:true}`、`Edge::Up` で `down:false` を返す
  （＝`Resolved::Fire` を **upでも返す**必要がある。今は up が一律 `Ignored`）
- adapter は `press()` / `release()` を別々に呼ぶ（`send_key()` の分解）
- **離し忘れ対策が必須**: WS切断・ページ非表示・pointercancel のときにHub側で
  押しっぱなしのキーを全部 release する。これを入れないと**キーが押されっぱなしでPCが操作不能になる**

### 3.4 リストは新Componentか、Deckの描画違いか

**Deckの描画違いを推す。** データはどちらも「label ＋ action の並び」で完全に同じ。
違うのは見た目だけ（正方形の格子 か、横いっぱいの縦リストか）。

- 同じにする利点: フォーマット1つ・ローダー1つ・WSメッセージ1つ（`deck.press`）で済む。
  「スロットの数を設定で変える」GUIも1つで両方に効く
- `component.render: "grid" | "list"` の1フィールドで切り替える

ユーザー提示のリストはこうなる（`decks/deck_story_paths.json`）:

```json
{
  "deckId": "story_paths",
  "grid": { "cols": 1, "rows": 2 },
  "pages": [ { "id": 1, "slots": [
    { "slotId": "S01", "label": "00_inbox",
      "action": { "t": "text", "string": "C:\\Users\\enjoy\\Dropbox\\09_Story\\01_hazimarinomatiboueisen\\00_inbox" } },
    { "slotId": "S02", "label": "01_storyRule",
      "action": { "t": "text", "string": "C:\\Users\\enjoy\\Dropbox\\09_Story\\01_hazimarinomatiboueisen\\01_StoryRule" } }
  ] } ]
}
```

---

## 4. 🚩 CLAUDE.md 不変条件6との関係（要裁定）

> 6. 書き込み系APIは **`keymaps/layers/` 配下**＋スキーマ検証＋`.bak`バックアップ付きのみ（D22）

要望6（配置をGUIでパズルのように変える）は、**`layouts/` 配下への書き込み**を必要とする。
現在の条文は書き込み先を `keymaps/layers/` に限定しているので、**条文の拡張が要る**。

推す形（D22の枠を広げるだけで、緩めない）:

- 書き込み先の許可リストを `keymaps/layers/` と **`layouts/`** の2つにする（他は不可のまま）
- スキーマ検証＋`.bak`バックアップは同じ条件で必須
- ファイル名はHubが決める（クライアントは `layoutId` しか送れない）＝任意パス書込にはならない

---

## 5. 段階分割の提案

| 段階 | 内容 | 依存 |
|---|---|---|
| **A** | `deck.grid.rows` を実際に使う（§1.3の欠陥修正）＋ Deck縦2段 ＋ **コピペリスト**（`render:"list"`） | 裁定不要。既存アクションのみ・新プロトコルなし |
| **B** | `layouts/*.json` ＋ `/layout` 面（JSON直編集・`/api/reload`で即反映） | §2の三層モデルの承認 |
| **C** | **十字キー**（`key.hold` ＋ 離し忘れ対策） | `key.hold` の承認 |
| **D** | `/settings` のドラッグ配置GUI | **不変条件6の拡張**（§4）が必須 |

Aは今日の裁定なしで着手できる（既存の器の中で完結するため）。

## 6. 受け入れ基準案（G形式）

- **G-a** `decks/deck_story_paths.json` を置いて再読込すると、リストが縦2件で出て、
  押すとPC側の入力欄にフルパスがそのまま入る（クリップボードは書き換わらない）
- **G-b** `deck_default.json` の `grid.rows` を 2 にして再読込すると、Deckが確かに2段で出る
  （今は `rows` が無視されるため、この基準が §1.3 の欠陥の回帰テストになる）
- **G-c** `layouts/layout_ipad_main.json` の section の `col` を書き換えて再読込すると、
  部品の位置が入れ替わる（HTMLは1行も触らずに）
- **G-d** 十字キーを押している間だけPC側でキーが押され続け、離すと止まる
- **G-e** 十字キーを押したままWSを切断すると、Hub側が押しっぱなしのキーを解放する
  （**PCが操作不能にならない**）
- **G-f** `cargo test --workspace` 全pass。凍結領域への差分ゼロ

## 7. 裁定

**裁定日: 2026-09-05 ／ 裁定者: ユーザー（AskUserQuestionによる直接回答）**

- [x] §2 三層（section / Component / slot、**1 section = 1 Component**）で進める
- [x] §3.1 section は**自由グリッド**（row/col/colSpan/rowSpan）にする。固定の名前付き区画は採らない
- [x] §3.3 **`key.hold` を追加してよい**。ただし切断・非表示時にHubが強制的に解放する仕組みを同時に入れること
- [x] §4 **不変条件6の書き込み許可に `layouts/` を追加してよい**（スキーマ検証＋`.bak`必須・ファイル名はHubが決める、は維持）
- [x] §5 **段階A から着手**する
- [ ] §3.4 リストをDeckの描画違いにするか（段階Aで `render:"grid"|"list"` として実装し、実機で見てから最終確認）

## 8. 進捗

| 段階 | 状態 |
|---|---|
| **A**（`grid.rows`の欠陥修正・Deck縦2段・コピペリスト） | 着手（2026-09-05） |
| B（`layouts/*.json` ＋ `/layout` 面） | **完了（2026-09-05・T24）** |
| C（十字キー・`key.hold`） | **完了（2026-09-05・T24）**。切断時の強制解放も実装・実証済み |
| D（配置GUI） | 未着手。不変条件6の条文更新が必要。**当面は `docs/HUB_MANUAL.md` の手順でJSON編集** |

### 段階Aで判明した追加の必要事項

Hubは**Deckを1枚しかロードしていない**（`main.rs` の `DECK_PATH = "decks/deck_default.json"` 固定）。
コピペリストはDeckの描画違いとして作るため、**Deckを複数ロードできるようにする**必要がある
（`keymaps/` と同じディレクトリスキャン方式へ揃える）。これに伴い:

- `surface.config` の `deck`（単数）→ `decks`（`deckId`をキーにしたマップ）
- `deck.press` に **`deckId` を追加**（省略時 `"default"`）。deckIdもslotIdと同じ「位置ID」なので
  不変条件1（クライアントが送ってよいのは位置IDのみ）に抵触しない。
  これによりslotIdはDeckごとにローカルでよくなり、ユーザーが全Deckを通して一意な
  slotIdを考える負担が無くなる
