# P-003 Stream Deck v2（アクション拡張＋分割画面）

起票: 2026-09-05 ／ 起票者: Claude Code（ユーザー着想の受け）
状態: **§7・§11は裁定済み（2026-09-05）／Ver1-a実装完了。§5細部・§8-1・§8-4・§8-6は未裁定**（→ §12・§13）

見た目の正（先に作成済み）: `brief/mockup/screen_mock_streamdeck_v0.8.html`
流用元: `brief/mockup/screen_mock_v0.4.html`（配色トークン・13列キーボード配置）

---

## 1. 対象アプリ/画面

APP_KeyDeck01。既存の Deck 面（`/deck`・`static/deck.html`・`decks/deck_default.json`）を
**Stream Deck v2** に拡張し、あわせて **分割面（上=Deck／下=キーボード）を新設**する。

ユーザー要望（原文の要点）:

1. Elgato Stream Deck ソフトの登録用アクション一覧（システム8種＋マルチアクション4種＋
   折りたたみのStream Deck/サウンドボード）と同等の機能を作りたい
2. 画面分割を可能にして、**下をキーボード・上をStream Deck**にしたい
3. 渡したリストは**登録用（設定画面）**のもの。**表示側は四角いスロット**にする

## 2. 課題

現状のDeck面でスロットに置けるのは `key` / `chord` / `text` / `keymap.switch` /
`keymap.reset` / `none` の6種だけで、**「PC上で何かを起こす」出口が一切ない**。
「OBSを立ち上げて、1.5秒待って、配信開始のホットキーを送る」のような、
Stream Deckを買う理由そのものの操作が1枚のボタンにならない。

また現状は面が1枚1URL（`/kb` `/deck` `/ipad` `/trackball`）で、
**Deckとキーボードを同時に出せない**。実際の使い方は「文章を打ちながら、時々ボタンを押す」で、
面を行き来するために毎回URLを切り替えるのは現実的でない。

## 3. 提案Surface（既存部品の流用を最優先）

**新規の面は1枚だけ（分割面）。Deck面・キーボード面の中身は流用する。**

流用するもの:

- WS接続・token認証・`surface.config` 配信・D9エラー書式 … 既存の `deck.html` / `ipad.html` の骨格
- スロット描画・`deck.press` 送信 … `deck.html` の `render()` / `sendPress()`
- 13列グリッドキーボード … `ipad.html`（配置の正は `screen_mock_v0.4.html`）
- JSON駆動 … `decks/deck_*.json` に列・行・ページ・スロットを書く既存の形をそのまま拡張

新規発明は次の3点だけに絞る:

1. **アクション型の追加**（§4）
2. **PC側リソース登録簿**（§5。`apps/apps.json`）
3. **分割面のレイアウト**（§6）

## 4. 必要な新規要素と実現可能性

### 4.1 アクション対応表（ユーザー提示リスト → JSON `t`）

| 登録画面での名前 | `t` | 状態 | 備考 |
|---|---|---|---|
| ホットキー | `chord` | **既存** | 追加実装なし |
| テキスト | `text` | **既存**（D20） | 追加実装なし |
| マルチメディア | `key`（`MUTE`/`VOL_*`/`MEDIA_*`） | **既存** | 新規型は不要。登録画面のプリセットとして見せるだけ |
| Webサイト | `web.open` | 新規 | `{ t:"web.open", url }` |
| ホットキーの切り替え | `chord.toggle` | 新規 | `{ t:"chord.toggle", a:[…], b:[…] }` |
| 開く（ファイル/フォルダ） | `shell.open` | 新規 **🚩** | `{ t:"shell.open", pathId }` |
| アプリケーションを開く | `app.launch` | 新規 **🚩** | `{ t:"app.launch", appId }` |
| 閉じる | `app.close` | 新規 **🚩** | `{ t:"app.close", target }` |
| マルチアクション | `multi` | 新規 | `{ t:"multi", steps:[…] }` |
| マルチアクションのスイッチ | `multi.toggle` | 新規 | `{ t:"multi.toggle", a:[…], b:[…] }` |
| ランダムアクション | `random` | 新規 **🚩** | `{ t:"random", choices:[…] }` |
| Key Logic | `logic` | 新規 | `{ t:"logic", steps:[…] }`（`if` / `delay`） |
| プロファイル切替 | `keymap.switch` | **既存** | Stream Deckカテゴリに再掲するだけ |
| 既定に戻す | `keymap.reset` | **既存** | 同上 |
| ページ切替 | `deck.page` | 新規 | `{ t:"deck.page", to }`。Hub状態でなくクライアント内で完結してもよい |
| 効果音の再生 | `sound.play` | **見送り** | v0.4のマクロ枠と同じ「場所だけ確保」。§8参照 |

**🚩** = 裁定が必要（§5・§7）。

### 4.2 D5・D28に対する適合

これらは全て**PC側のJSONに書かれた定義**であり、クライアントが送るのは今までどおり
`{"type":"deck.press","slotId":"S09"}` の**位置IDだけ**。
不変条件1「クライアントが送ってよいのは位置IDか正規化状態のみ」は**維持される**。
新しいWS APIは1本も足さない。

### 4.3 実装量の見積り

| 変更先 | 内容 |
|---|---|
| `crates/proto-keymap/src/lib.rs` | `Action` に新バリアント追加（`deck.rs` / `ws.rs` / adapter の exhaustive match が全て波及。T15と同じ手順） |
| `crates/proto-hub/src/deck.rs` | 新アクションのロード時検証（URLスキーム・`appId` 参照先の存在・`multi` の入れ子禁止・`steps` 上限） |
| 新規 `crates/proto-hub/src/apps.rs` | `apps/apps.json` のロード＋許可リスト（`surface.rs` と同じ構造） |
| `crates/proto-hub/src/ws.rs` | `handle_deck_press` を「1アクション発火」から「アクション列の逐次実行」へ（`multi`/`logic` の delay は tokio タイマ） |
| `crates/proto-adapter-win/src/lib.rs` | `web.open` / `shell.open` / `app.launch` / `app.close` の実行（§7の防御込み） |
| `static/panel.html`（新規） | 分割面 |
| `static/deck.html` | 四角スロット化（CSS のみ。`aspect-ratio:1`） |
| `static/settings.html` | 登録画面（右サイドバー＋プロパティ）。**T9（VIAL型GUI）と同じ器**なので、ここは1本にまとめるか要判断 |
| `schemas/deck.schema.json` / 新規 `schemas/apps.schema.json` | 書式の正 |

## 5. 中核の設計判断: パスをDeck JSONに直接書かない

D28で確立した「**クライアントは何に繋がるかを指定できない。出口はHub側のJSONだけが決める**」を
1段先に進め、**Deck JSON にも生パスを書かない**ことを提案する。

```
[Deck JSON]                    [登録簿 apps/apps.json]        [実行]
{ t:"app.launch",  ──appId──▶  { "obs": {                ──▶  CreateProcess
  appId:"obs" }                    "exec": "C:\\…\\obs64.exe",
                                   "args": ["--startvirtualcam"],
                                   "cwd":  "C:\\…\\bin64" } }
```

理由:

- Deck JSONは「配布・共有される想定のファイル」（D11: Import=フォルダに置いて再起動）。
  他人のセットリストを取り込んだ瞬間に任意のexeパスが走る形は避けたい
- 登録簿を1枚に集約すれば、「このHubは何を起動しうるか」を**1ファイル読むだけで棚卸しできる**
- `surfaces/*.json` と同じ形なので、実装も読み方も既存の延長で済む

同じ理由で `shell.open` も `pathId` 参照とする。
`web.open` の `url` だけはDeck JSONに直接書く案を推す（URLは実行ファイルではなく、
スキーム許可リストで十分に絞れるため）。**この非対称を許すかも裁定対象**。

## 6. 分割面のレイアウト

- URL: `/panel?token=…`（既存4面はそのまま残す。分割面は5枚目）
- 上ペイン=Deck（四角スロット・ページドット）／下ペイン=キーボード（13列グリッド）
- 比率は CSS変数1つ（`grid-template-rows: var(--split) 8px var(--splitkb)`）で切替。
  Ver1は3プリセット（デッキ大／半々／キーボード大）。ドラッグでの無段階調整はVer2送り
- **WSは1本**。`surface.config` は既に `keymap` と `deck` の両方を1メッセージで配っているので、
  分割面はプロトコル変更ゼロで成立する（=既存の器がそのまま使える）
- 罠: CSS Grid の `1fr` は内容の最小幅を下回れない（STATE.md「落とし穴」）。
  上下分割でも同じ罠があるため、行トラックは `minmax(0, …)` を徹底する

## 7. 🚩 CLAUDE.md 不変条件6との関係（**要裁定・最重要**）

> 6. 書き込み系APIは `keymaps/layers/` 配下＋スキーマ検証＋`.bak`バックアップ付きのみ（D22）。
>    **任意パス書込・任意コマンド実行APIは絶対に作らない**

`app.launch` / `shell.open` / `app.close` は、この条文の**文言には触れないが、精神には触れる**。

- **触れない理由**: 条文が禁じているのは「任意〜**API**」＝クライアントが中身を指定できる口。
  今回はクライアントが送るのは `slotId` のみで、実行内容はPC側の2枚のJSON（Deck＋登録簿）が
  100%決める。新しいWS APIも増えない
- **それでも触れる理由**: 「WSに繋げた者は、登録済みアプリを起動できる」状態にはなる。
  token（D8）が漏れた場合の被害の質が「キー入力の送信」から「プロセス起動」へ上がる

D28で採った判断基準（=業界標準の形か／既にできることと比べたリスクの実質的増分）を当てると:

- 既に `chord: ["WIN","R"]` ＋ `text: "obs64.exe"` ＋ `key: "ENTER"` を1スロットに並べれば、
  **今でも実質的に任意アプリは起動できる**（`multi` が無いので3スロット必要というだけ）。
  リスクの実質的増分は小さい
- Elgato Stream Deck を含む同種製品はすべてこの機能を持つ。業界標準から外れた設計ではない

**したがって「採用」を推すが、次の防御を同時に入れることを条件とする:**

| # | 防御 | 内容 |
|---|---|---|
| A | 登録簿方式 | §5。Deck JSONに生パス・生コマンドを書けない |
| B | シェルを経由しない | `cmd /c` / `powershell -c` / `ShellExecute` の文字列連結は使わず、`exec`＋`args` 配列を `CreateProcess` 相当へ直接渡す。**シェルのメタ文字が意味を持つ経路を作らない** |
| C | ロード時検証 | 起動時に `exec` の実在確認・拡張子許可リスト（`.exe` `.lnk` `.bat`は不可）。違反は `LOAD_APPS_*` ＋cause で**起動拒否**（既存のロード検証と同じ強さ） |
| D | URLスキーム許可 | `web.open` は `http` / `https` のみ。`file:` `javascript:` `ms-…:` 等は拒否 |
| E | `app.close` の既定 | 既定は `target:"active"`（＝ALT+F4相当、今でもchordで送れる）。プロセス名指定の強制終了は**Ver1では作らない** |
| F | 実行ログ | 発火のたびにD9書式で1行（`chk`・appId・exec）。何が起動したか後から追える |
| G | 同時実行の抑制 | `multi` の実行中に同じスロットが再押下されたら無視（多重起動の暴発防止） |

**裁定していただきたいのはこの1点です:**
上記A〜Gを条件に `app.launch` / `shell.open` / `app.close`（active限定）を採用してよいか。
不可の場合は、`web.open` と `multi` / `chord.toggle` / `logic`（＝既存アクションの合成のみ）に
絞ったVer1でも、Stream Deckとしての価値の6割は出せる。

## 8. その他の要裁定事項

| # | 論点 | 推奨 |
|---|---|---|
| 8-1 | **`random` とG5の衝突** — 「同じ入力列は常に同じ結果」（決定性）はレイヤー解決エンジンの不変条件。`random` はDeck発火なのでエンジンは通らないが、アプリ全体の性格としては初めての例外になる | ダイス・抽選という用途は本物のStream Deckにもある。**採用するがDeckスロット限定**とし、キーマップ側（`layers/*.json`）には置けないようスキーマで禁じる |
| 8-2 | **`logic` の条件式の範囲** | 任意式は評価しない。事前登録の述語のみ（`foreground.is` / `layer.is` / `keymap.is`）。Ver1は `foreground.is` だけでも成立 |
| 8-3 | **`multi` の入れ子** | 禁止（無限ループ防止）。`steps` に置けるのは単発アクション＋`delay` のみ。`steps` 上限32・`delay` 上限10秒をロード時に検証 |
| 8-4 | **設定画面の器** | 本提案の登録画面と、未着手のT9（VIAL型キーマップ編集GUI）は同じ `static/settings.html` に同居する。**1画面2タブに統合するか、別ページに割るか** |
| 8-5 | **`sound.play`** | Hub機（PC）で鳴らすのか端末（タブレット）で鳴らすのか未決。Ver1では見送り、UIに枠だけ置く（v0.4のマクロ枠と同じ扱い） |
| 8-6 | **`icon`** | 既存スキーマは `icon: data URI` を許すが `deck.html` は `<img>` を出すだけ。モックは絵文字1文字で代用した。**絵文字を正式に認めるか（`icon` が1〜2文字なら文字として描画）** |

## 9. 受け入れ基準案（G形式）

- **G-a** `decks/deck01.json` に `multi`（`app.launch`→`delay`→`chord`）を1スロット書き、
  スマホから1回押すと、PC上でアプリが起動し、指定ms後にホットキーが届く
- **G-b** 登録簿に無い `appId` を書いたDeck JSONを置くと、Hubが `LOAD_APPS_UNKNOWN_ID` ＋
  cause（どのファイルのどのslotIdか）を出して**起動を拒否**する
- **G-c** `web.open` に `file:///C:/` を書くと `LOAD_URL_SCHEME_DENIED` で起動を拒否する
- **G-d** `apps.json` の `exec` に `.bat` を書くと起動を拒否する
- **G-e** `/panel?token=…` を開くと上にDeck・下にキーボードが同時に出て、
  **キーボードのレイヤー状態とDeckの発火が同一WS 1本で両立**する（片方が他方を壊さない）
- **G-f** 分割面の比率3プリセットが効き、キーボード大でも13列が横に収まる（`minmax(0,1fr)` の罠の回帰確認）
- **G-g** 表示側のスロットが**常に正方形**（`aspect-ratio:1`）で、5列3行・4列2行など
  `grid` 指定を変えても崩れない
- **G-h** `cargo test --workspace` が全pass（現在95件。既存テストの削除・弱体化なし）
- **G-i** 凍結領域（`crates/hub-core/`・`keymaps/keymap_default.json`・`brief/` の既存設計書）への差分ゼロ

## 10. 昇格条件5項目の自己評価（vision §2）

| # | 項目 | 評価 |
|---|---|---|
| ① | MVP完成 | △ 見た目モックのみ完成。実装は未着手（本提案の裁定待ち） |
| ② | 利用場面が1つに定まる | ○ 「タブレット1枚で、文章を打ちながら配信・作業のトリガーも押す」 |
| ③ | 受け入れ基準を先に書ける | ○ §9のG-a〜G-i |
| ④ | 既存Protocol内か | ○ **WS APIの追加ゼロ**。`deck.press` と `surface.config` のまま成立 |
| ⑤ | 10日以内の粒度 | △ **全部入りだと超える**。§11の3段階に割ることを提案 |

## 11. 段階分割の提案（⑤への回答）

| 段階 | 内容 | 依存 |
|---|---|---|
| **Ver1-a** | 分割面 `/panel` ＋ 四角スロット化。**アクションは既存6種のまま** | 裁定不要（見た目と器だけ） |
| **Ver1-b** | 既存アクションの合成のみ: `multi` / `multi.toggle` / `chord.toggle` / `deck.page` / `web.open` | §7の裁定のうちDのみ |
| **Ver1-c** | PC操作系: `shell.open` / `app.launch` / `app.close` ＋ 登録簿 ＋ `logic` / `random` | §7・§8-1の裁定が必須 |

Ver1-a は本提案の裁定を待たずに着手できる（既存の不変条件に一切触れないため）。

## 12. 裁定

**裁定日: 2026-09-05 ／ 裁定者: ユーザー（AskUserQuestionによる直接回答）**

- [x] §7 `app.launch` / `shell.open` / `app.close` を**防御A〜G条件付きで採用**する
  → A〜G（登録簿方式／シェル非経由／ロード時の実在・拡張子検証／URLスキーム許可リスト／
    `app.close`はactive限定／D9実行ログ／多重発火抑制）は**実装の必須条件**であり、
    1つでも省く場合は再裁定が要る
- [x] §11 段階分割（Ver1-a／1-b／1-c）を採用し、**Ver1-a から先に着手**する
- [ ] §5 登録簿方式の細部（`web.open` の `url` 直書きという非対称を許すか）— Ver1-c着手時に確認
- [ ] §8-1 `random` の採否（Deckスロット限定案）— Ver1-c着手時に確認
- [ ] §8-4 登録画面をT9（VIAL型GUI）と同居させるか — Ver1-c着手時に確認
- [ ] §8-6 `icon` に絵文字を認めるか — Ver1-b着手時に確認

## 13. 進捗

| 段階 | 状態 | 記録 |
|---|---|---|
| **Ver1-a** | **完了（2026-09-05）** | `static/panel.html` 新規・`static/deck.html` 四角スロット化・`/panel` ルート＋QR target 追加。`DEVBOARD.md`「Stream Deck v2 Ver1-a」節に検証記録 |
| Ver1-b | 未着手 | `multi` / `multi.toggle` / `chord.toggle` / `deck.page` / `web.open` |
| Ver1-c | 未着手 | `shell.open` / `app.launch` / `app.close` ＋ 登録簿 ＋ `logic` / `random` |

### Ver1-a で確定した設計判断（Ver1-b以降もこれを踏襲する）

1. **分割面はWS 1本・`surface=ipad` で接続する**。`surface.config` が元から
   `keymap` と `deck` を同一メッセージで配っているため、**プロトコル・状態管理・
   `SurfaceKind` の追加はゼロ**で成立した。新しい `SurfaceKind::Panel` は作っていない
   （作ると broadcast 経路を全て触ることになり、得るものが無い）
2. **四角スロットの大きさは「上ペインの高さ」から逆算する**。四角形は幅で大きさが
   決まるため、行数の多いDeck（5列×3行）をそのまま置くと上ペインからはみ出す。
   1マスの下限は **44px**（タップ目標の下限。横向きスマホの「半々」で実測35pxまで
   縮んだため）。下限に達したら縮めず上ペインをスクロールさせる
3. **ページドットは `pages` が2枚以上のときだけ出す**。表示専用でHubへは何も送らない
   （`deck.page` アクションはVer1-bの範囲）
