# REQ-20261001-001 への返事 — Note Story 起動ボタン1つで、PC で開き専用の盤面へ移る（Hooks）

```
返答日    2026-10-01
担当      KeyDeck 統括チャット（Claude Code / Opus 5.5）
状態      作成済み。本物の Hub と PC のブラウザで確認。iPad 実機ではまだ
```

## (1) 作ったもの

**データ**
- `apps/apps.json` の `note_story` を、インストール済みの PWA を開く形にした
  `chrome_proxy.exe --profile-directory=Default --app-id=addplacbfmomdnicgdfhmekgmlgfakhg`
  （スタートメニュー「Chrome アプリ」フォルダーの Note Story ショートカットと同じ引数。AI が調べて入れた）
- 入口のボタン: キーの組 `appswitch`（盤面 `shortborad` に置いてある）に **A3「Note Story 起動」** を足した
  `{"t":"app.launch","id":"note_story","fire":{"t":"layout.switch","id":"note_story"}}`
- note_story の盤面の「Note Story 開く」（note_edge E5）は、依頼どおり**ただ起動のまま**

**Rust**（利用者の裁定: 2026-10-01「広げるでいいよ」）
- B-002 の修正（下の (2)）
- 読み込みの決まりを1つ広げた: **app.launch の fire に限り、fire を持たない layout.switch を置ける**。盤面を移るのは画面の話で PC へは何も送らないので、送れるものは増えない。入れ子は2段まで
- ついでに塞いだ穴: **Deck の app.launch の fire を、これまで何も検査していなかった**（どんな入れ子でも読み込めた）。キーボード側と同じ規則にした
- 読み込み時の確認を1つ足した: layout.switch の移る先が**実在する盤面か**（入れ子も含む）。これまでは押したときに初めて分かっていた

## (2) B-002 の修正内容とテスト

- 起動時に許可リストを作るとき、**入れ子の fire を全部たどる**ようにした（`startup.rs` の `with_nested`。tg.fire・layout.switch・app.launch の3種）
- テスト（新規5本。`cargo test --workspace` = **178 passed**、既存の削除・弱体化なし）
  - `nested_fire_of_every_kind_reaches_the_allow_list` — **3種すべて**で内側が許可リストに載る（app.launch 自身も載ったまま）
  - `only_app_launch_may_fire_a_plain_layout_switch` — 許すのは app.launch の中の fire なし layout.switch だけ。layout.switch の中の layout.switch、tg.fire の中、2段を超える入れ子、app.launch の中の app.launch / mo は拒否
  - `app_launch_fire_on_deck_is_checked` — Deck 側も同じ規則
  - `layout_switch_to_unknown_board_is_rejected_even_when_nested` — 実在しない盤面は読み込みで止まる
- 実データの読み込み（appswitch・note_story・apps など全部）もテストで通る
- 補足: appswitch の A1（YouTubeへ・Win+1）は、B-002 の前から実は動いていた可能性が高い（Deck `apps` が Win+1 を持っていて、そちらから許可リストに載っていたため）。修正後は A1 自身の定義からも載る。**A1 は押していない**（押すと PC のタスクバー1番目が前に出るため）

## (3) すでに開いているとき窓が増えるか — **増える**

2026-10-01 に実測（`--app-id` で2回起動 → Note Story の窓が 1 → 2 → 3）。
Note Story の manifest に `launch_handler` が無いため、Chrome は起動のたびに新しい窓を開く。

**増やさない案（Note Story 側の変更。こちらは触っていない）**
`vite.config.ts` の VitePWA の manifest に次を足してデプロイ → Chrome で PWA を入れ直さなくても、次の更新で反映されるはず。
```json
"launch_handler": { "client_mode": "focus-existing" }
```
`focus-existing` は「開いている窓があればそれを前に出す」。KeyDeck 側で窓を探して前に出すには Hub が画面を見る機能が要り、それは作らない方針なので、manifest で解くのが筋。

※ 確認のために開いた Note Story の窓が PC に残っている（試験で2つ増やした）。閉じてかまわない。

## (4) `--app=URL` と `--app-id` のどちらにしたか — **`--app-id`**

- インストール済みの PWA として開くので、アイコン・タスクバーのまとまり・窓の見た目が PWA のものになる
- 上の `launch_handler`（窓を増やさない）は、**インストールした PWA を app-id で開いたときに効く**。`--app=URL` のままだとただのアプリ窓で、効かない
- 注意: **PWA を入れ直すと app-id が変わる**。そのときは apps.json の `--app-id=` を書き換えて Hub を再起動（手順は下）

## (5) 起動に失敗したときの動き

起動に失敗したら（chrome_proxy.exe が無い、app id が apps.json に無い など）、**盤面は移らない**。押した端末へエラーを返す（D9: code + cause。例 `APP_LAUNCH_FAILED` / `APP_LAUNCH_UNKNOWN`）。起動→盤面の順は Hub のコードで決まっている（`ws.rs` の app.launch は失敗したら fire を撃たずに戻る）。

## 確かめたこと（本物の Hub・PC のブラウザ2枚）

- iPad を名乗る画面（盤面 shortborad）で **A3 を実マウスで押す** → PC で Note Story が開き、**iPad の画面だけ** note_story へ移った。Pixel を名乗る画面は game_iphone7_port のまま
  - Hub ログ: `app launched app=note_story` → `layout switch; this device only device="ipad" sent=1`
- apps.json に無い id・exe/args を端末へ出さない・.bat/.cmd/.ps1 の拒否は変えていない（既存テストがそのまま通る）

## まだのこと・決めてほしいこと

- **iPad 実機での通し**（手順書の5）
- **「戻る(盤)」の行き先**: note_story の「既定へ戻る」は、押した端末の既定（devices.json の defaultLayout）へ戻る。いま iPad の既定は **note_story** にしてあるので、**戻っても note_story のまま**になる（受け入れ基準の「入口の盤面へ戻れる」を満たしていない）。どちらかに決めてほしい
  - A. iPad の既定を入口の盤面（例 shortborad）にする → QR から開くと入口が出て、A3 で Note Story へ
  - B. 「既定へ戻る」を「入口へ」（`layout.switch` の id に入口の盤面を明示）に変える → iPad の既定は note_story のまま
- 入口の盤面が `shortborad` でよいか（ほかの盤面にも A3 を置くなら、置く場所を教えてほしい）

## 人が手でやる初期設定（手順書）

1. **PWA のインストール** — 済み（2026-10-01 利用者）。app-id は AI が調べて apps.json に入れた
2. **Hub の再起動** — `apps.json`・`devices.json` は起動時にだけ読む。書き換えたら `start_hub.cmd` で起動し直す
3. **端末で入口 URL を開き直す** — Hub を起動し直すと token が変わる。PC のトップ画面の「QR」ボタン（何も変更していないときの保存ボタン）で各端末の QR が出る
4. **F13〜F17** — デプロイ済みなので「押して登録」は不要のはず。効かなければ Note Story の 設定 → ショートカット → PC タブで確認
5. **iPad 実機で通し** — 入口の盤面（shortborad）の A3「Note Story 起動」を押す → PC に Note Story の窓が出る・iPad の盤面が note_story に変わる
6. **PWA を入れ直したとき** — スタートメニュー「Chrome アプリ」→ Note Story ショートカットのプロパティの「リンク先」にある `--app-id=…` を、apps.json の `note_story` の args へ写して Hub を再起動

## 追記（2026-10-01）: 「戻る(盤)」の行き先 — 利用者が A に決定

- `devices/devices.json` の iPad の `defaultLayout` を **`shortborad`（入口の盤面）** にした。QR や端末の入口 URL から開くと入口が出て、A3「Note Story 起動」で Note Story へ移る
- 本物の Hub で確認: iPad を名乗る画面（note_story）の「既定へ戻る」を実マウスで押す → **shortborad に戻った**
- guardian 点検（b3ba42f）: **PASS**（reports/guardian_20261001_0029.md）。起動に失敗したとき盤面が移らないことはコードを読んでの確認で、テストは次に手を入れるときに足す
