// ============================================================
// components.js — 部品（キーボード / Deck）の描画。**唯一の実装**。
//
// ■ なぜ切り出したか
//   実機の面（layout.html）と区画エディタ（editor.html）が、同じ部品を
//   別々のコードで描いていると必ずズレる。ズレた瞬間、エディタのプレビューは
//   嘘になり、「正しく置けているか」を確かめる役に立たなくなる。
//   このリポジトリでは実際に同じ形の事故が起きている（接続先の一覧が
//   main.rs と ws.rs に手書きで散り、レイアウト3種がコンソールに出ていなかった。
//   T31 で connection_targets() に集約して解消）。
//
//   したがって **描画はここにしか書かない**。読み込む側は
//   <script src="/components.js"></script> を置いて KDComponents を呼ぶ。
//
// ■ 2つのモード
//   interactive: true  … 実機の面。押下でHubへ送る（送信はホスト側のコールバック）
//   interactive: false … Hubへは送らない
//   editable: true     … Hubへは送らないが、**触れる**（編集画面用）
//
//   「Hubへ送らない」と「触れない」は別のこと。まとめて扱うと、編集画面の盤面まで
//   pointer-events: none になり、指でもマウスでも一切反応しなくなる。
//   実際にそうなっていた。合成イベントでの確認は当たり判定を素通りするので気づけない。
//
//   **送信そのものはここに書かない。** 何を送るかはホストが決める（不変条件1）。
//   ここが持つのは「JSONをDOMにする」ところまで。
//
// ■ CSSも持つ
//   見た目が2箇所に分かれるとやはりズレるので、CSSもここが持って1度だけ挿す。
//   色は var(--panel, 既定値) の形にしてあり、ホストが定義していればそちらが勝つ。
// ============================================================
(function (global) {
  "use strict";

  var STYLE_ID = "kd-components-style";

  var CSS = [
    ".kd-surface {",
    "  --kd-panel: var(--panel, #1c2338);",
    "  --kd-ink: var(--ink, #f4f7ff);",
    "  --kd-ink-muted: var(--ink-muted, #b7bfd6);",
    "  --kd-ink-dim: var(--ink-dim, #4b5468);",
    "  --kd-empty: var(--empty, #161c30);",
    "  --kd-accent: var(--accent, #2d67d9);",
    "  --kd-accent-soft: var(--accent-soft, #24478f);",
    "  --kd-ok: var(--ok, #35c47a);",
    "  --kd-line: var(--line, #2d3550);",
    "  --kd-key-radius: var(--key-radius, 10px);",
    "  --kd-key-gap: var(--key-gap, 6px);",
    "  --kd-slot-radius: var(--slot-radius, 12px);",
    "  min-width: 0; min-height: 0;",
    "}",

    // ---- テーマ ----
    // 各画面は :root に自分の色を持っている。ここはその**上書き**なので、
    // 属性つきの :root で書いて必ず勝たせる（同じ強さだと読み込み順で揺れる）。
    //
    // 既定（blue）は各画面の :root をそのまま使う＝ここでは何も書かない。
    // 色を1箇所にまとめ直すのは、全画面の見た目を一度に変える大きな作業になるため、
    // まず「赤へ切り替わること」を先に通す。
    ":root[data-kd-theme=\"red\"] {",
    "  --bg: #1c1a1c; --panel: #2a2628; --panel2: #221f21;",
    "  --ink: #f6f1f2; --ink-muted: #cabfc2; --ink-dim: #8b7f83;",
    "  --accent: #e23a3f; --accent-soft: #6f1e21; --line: #3b3437;",
    "  --empty: #201d1f; --key-radius: 12px;",
    "}",
    // 赤テーマは参考画像と同じく「厚みのある黒い板」に寄せる。
    // 平らな面に赤を置くだけだと、ただ色が変わっただけに見える。
    ":root[data-kd-theme=\"red\"] .kd-surface .key {",
    "  box-shadow: inset 0 1px 0 rgba(255,255,255,.06), 0 2px 6px rgba(0,0,0,.55);",
    "}",
    ":root[data-kd-theme=\"red\"] .kd-surface .key.pressed,",
    ":root[data-kd-theme=\"red\"] .kd-surface .key.pressing {",
    "  box-shadow: inset 0 2px 8px rgba(0,0,0,.7);",
    "}",
    ":root[data-kd-theme=\"red\"] .kd-surface .key.fn { color: var(--kd-accent); }",

    // 緑。Unity開発中の盤面。赤と同じ「厚みのある黒い板」の作りに揃え、
    // 差し色だけを替える（作りまで変えると、同じ道具に見えなくなる）。
    ":root[data-kd-theme=\"green\"] {",
    "  --bg: #141a16; --panel: #1e2721; --panel2: #18201a;",
    "  --ink: #f1f7f2; --ink-muted: #bccabf; --ink-dim: #7d8d82;",
    "  --accent: #35b46a; --accent-soft: #1d5c39; --line: #334037;",
    "  --empty: #161d18; --key-radius: 12px;",
    "}",
    ":root[data-kd-theme=\"green\"] .kd-surface .key {",
    "  box-shadow: inset 0 1px 0 rgba(255,255,255,.06), 0 2px 6px rgba(0,0,0,.55);",
    "}",
    ":root[data-kd-theme=\"green\"] .kd-surface .key.pressed,",
    ":root[data-kd-theme=\"green\"] .kd-surface .key.pressing {",
    "  box-shadow: inset 0 2px 8px rgba(0,0,0,.7);",
    "}",
    ":root[data-kd-theme=\"green\"] .kd-surface .key.fn { color: var(--kd-accent); }",

    // ---- サブheader（2段目） ----
    // 左に「いまどこに居るか」、右にその画面でしかやらない操作。
    // 画面ごとにボタンの居場所が変わると、毎回探すことになる。
    // 色はページ側の変数を直に見る（--kd-* は .kd-surface の中にしか無い）。
    ".kd-subhead {",
    "  position: relative; display: flex; align-items: center; gap: 10px;",
    "  flex-wrap: wrap; margin: 8px 0 10px; padding: 5px 12px; min-height: 34px;",
    "  background: var(--panel2, #161c2e); border: 1px solid var(--line, #2d3550);",
    "  border-radius: 8px;",
    "}",
    ".kd-crumbs { display: flex; align-items: center; gap: 6px; font-size: 12px; }",
    ".kd-crumbs a { color: var(--ink-muted, #b7bfd6); text-decoration: none; }",
    ".kd-crumbs a:hover { color: var(--ink, #f4f7ff); text-decoration: underline; }",
    ".kd-crumbs .sep { color: var(--ink-dim, #6b7590); }",
    ".kd-crumbs .here { color: var(--ink, #f4f7ff); font-weight: 700; }",
    ".kd-subactions { margin-left: auto; display: flex; align-items: center;",
    "  gap: 6px; flex-wrap: wrap; }",
    // 知らせはこの帯の**上に浮かせる**。1行を占めると、出るたびに下の中身が
    // 押し下げられ、キーの高さまで変わる（実際に一瞬動く不具合になっていた）。
    ".kd-toast {",
    "  position: absolute; left: -1px; right: -1px; top: -1px; z-index: 30;",
    "  margin: 0; padding: 7px 12px; border-radius: 8px; font-size: 12px;",
    "  background: #3a2f14; color: var(--warn, #e8b44a); border: 1px solid #6b4c17;",
    "  box-shadow: 0 8px 20px rgba(0,0,0,.5);",
    "  opacity: 0; transform: translateY(-4px); pointer-events: none;",
    "  transition: opacity .16s ease-out, transform .16s ease-out;",
    "}",
    ".kd-toast.show { opacity: 1; transform: none; }",
    ".kd-toast[hidden] { display: none !important; }",
    "@media (prefers-reduced-motion: reduce) { .kd-toast { transition: none; } }",

    // ---- 画面切り替え（看板ボタン＋自前のメニュー） ----
    ".kd-nav { position: relative; display: inline-block; }",
    ".kd-nav-btn {",
    "  display: inline-flex; align-items: center; gap: 7px;",
    "  background: var(--kd-panel); color: var(--kd-ink);",
    "  border: 1px solid var(--kd-line); border-radius: 8px;",
    "  font: inherit; font-size: 15px; font-weight: 700; letter-spacing: .01em;",
    "  padding: 4px 10px; min-height: 30px; cursor: pointer;",
    "}",
    ".kd-nav-btn:hover { border-color: var(--kd-accent); }",
    ".kd-nav-caret { font-size: 9px; color: var(--kd-ink-dim); transition: transform .15s ease; }",
    ".kd-nav-btn[aria-expanded=\"true\"] .kd-nav-caret { transform: rotate(180deg); }",
    // 開いた一覧は**明るい紙に黒文字**。背景色と文字色の両方をここで決め切る
    // （片方を環境まかせにすると、暗い板に暗い字が乗って読めなくなる）。
    ".kd-nav-menu {",
    "  position: absolute; left: 0; top: calc(100% + 6px); z-index: 200;",
    "  min-width: 210px; padding: 6px;",
    "  background: #f7f9ff; color: #10131c;",
    "  border: 1px solid #c9d2e6; border-radius: 10px;",
    "  box-shadow: 0 12px 28px rgba(0,0,0,.45);",
    "  transform-origin: top center;",
    "  transform: translateY(-6px) scaleY(.96); opacity: 0;",
    "  transition: transform .15s ease-out, opacity .15s ease-out;",
    "}",
    ".kd-nav-menu.open { transform: none; opacity: 1; }",
    ".kd-nav-menu[hidden] { display: none !important; }",
    ".kd-nav-group {",
    "  font-size: 10px; font-weight: 700; color: #5b6478;",
    "  padding: 7px 8px 2px; letter-spacing: .06em;",
    "}",
    ".kd-nav-item {",
    "  display: block; width: 100%; text-align: left;",
    "  background: transparent; color: #10131c; border: 0; border-radius: 7px;",
    "  font: inherit; font-size: 13px; padding: 7px 10px; cursor: pointer;",
    "}",
    ".kd-nav-item:hover { background: #dfe8fb; }",
    ".kd-nav-item.current { font-weight: 700; }",
    ".kd-nav-item[disabled] { color: #97a0b5; cursor: default; }",
    ".kd-nav-item[disabled]:hover { background: transparent; }",
    "@media (prefers-reduced-motion: reduce) {",
    "  .kd-nav-menu, .kd-nav-caret { transition: none; }",
    "}",

    // ---- ダイヤル（jog） ----
    ".kd-surface .jogwrap { display: flex; flex-direction: column; align-items: center;",
    "  justify-content: center; gap: 8px; height: 100%; min-height: 0; }",
    ".kd-surface .jogsvg { flex: 0 1 auto; min-height: 0; max-width: 100%; max-height: 78%;",
    "  touch-action: none; cursor: grab; aspect-ratio: 1; }",
    ".kd-surface .jogsvg:active { cursor: grabbing; }",
    ".kd-surface .jogtick { stroke: var(--kd-line); stroke-width: 2; stroke-linecap: round; }",
    ".kd-surface .jogtick.major { stroke: var(--kd-ink-dim); }",
    ".kd-surface .jogprog { fill: none; stroke: var(--kd-accent); stroke-width: 4; stroke-linecap: round; }",
    ".kd-surface .joglabels { display: flex; gap: 10px; align-items: center; font-size: 12px;",
    "  color: var(--kd-ink-muted); white-space: nowrap; }",
    ".kd-surface .joglabels b { color: var(--kd-ink); font-weight: 600; }",
    ".kd-surface.kd-preview .jogsvg { cursor: default; }",

    // ---- ホイール（jog の shape=wheel）。マウスのホイールを縦に置いた形 ----
    // 溝は repeating-linear-gradient で描き、指の移動に合わせて background-position を
    // ずらすだけで「回って見える」。上下の暗い帯で筒の丸みを出す。
    ".kd-surface .wheelwrap { display: flex; flex-direction: column; align-items: stretch;",
    "  gap: 4px; height: 100%; min-height: 0; }",
    ".kd-surface .wheellb { font-size: 10px; color: var(--kd-ink-muted); text-align: center;",
    "  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex: 0 0 auto; }",
    ".kd-surface .wheeldrum { position: relative; flex: 1 1 auto; min-height: 0;",
    "  width: min(100%, 64px); margin: 0 auto; border-radius: 14px; overflow: hidden;",
    "  border: 1px solid var(--kd-line); touch-action: none; cursor: ns-resize;",
    "  background-color: #1a1f2e;",
    "  background-image: repeating-linear-gradient(to bottom,",
    "    #3a4256 0px, #3a4256 3px, #1a1f2e 3px, #1a1f2e 9px); }",
    ".kd-surface .wheeldrum::after { content: ''; position: absolute; inset: 0; pointer-events: none;",
    "  background: linear-gradient(to bottom, rgba(0,0,0,.72), rgba(0,0,0,0) 30%,",
    "    rgba(255,255,255,.06) 50%, rgba(0,0,0,0) 70%, rgba(0,0,0,.72)); }",
    ".kd-surface .wheeldrum.on { border-color: var(--kd-accent); }",
    ".kd-surface.kd-preview .wheeldrum { cursor: default; }",

    // ---- ラジアルボタン（keymap に radial があるキーボード） ----
    // ボタン本体は区画の中。開いた選択肢（.kd-radialmenu）は区画の外へ出す必要があるので
    // body の直下に置き、position:fixed で画面の最前面に重ねる（区画の overflow:hidden に切られない）。
    ".kd-surface .radialbtn { position: relative; width: 100%; height: 100%; min-height: 0;",
    "  border: 1px solid var(--kd-line); border-radius: var(--kd-key-radius);",
    "  background: var(--kd-panel); color: var(--kd-ink); font: inherit; padding: 0;",
    "  display: flex; align-items: center; justify-content: center;",
    "  touch-action: none; user-select: none; cursor: pointer; }",
    ".kd-surface .radialbtn.on { background: var(--kd-accent); border-color: var(--kd-accent); }",
    ".kd-surface .radialbtn .rlabel { font-size: 13px; font-weight: 600; }",
    ".kd-surface .radialbtn .rarrow { position: absolute; font-size: 8px; color: var(--kd-ink-muted); line-height: 1; }",
    ".kd-surface .radialbtn .rarrow.n { top: 4px; left: 50%; transform: translateX(-50%); }",
    ".kd-surface .radialbtn .rarrow.s { bottom: 4px; left: 50%; transform: translateX(-50%); }",
    ".kd-surface .radialbtn .rarrow.e { right: 5px; top: 50%; transform: translateY(-50%); }",
    ".kd-surface .radialbtn .rarrow.w { left: 5px; top: 50%; transform: translateY(-50%); }",
    ".kd-radialmenu { position: fixed; inset: 0; z-index: 1000; pointer-events: none;",
    "  font-family: system-ui, -apple-system, 'Segoe UI', sans-serif; }",
    // 扇はSVGで描き、文字だけHTMLを上に重ねる（SVGのtextは折り返せないため）
    ".kd-radialmenu .rsvg { position: absolute; overflow: visible;",
    "  filter: drop-shadow(0 8px 22px rgba(0,0,0,.6)); }",
    ".kd-radialmenu .rseg { fill: rgba(26,33,54,.94); stroke: #39425f; stroke-width: 1; }",
    // 内周（細かい操作）は外周より一段暗くして、輪の違いを色でも分かるようにする
    ".kd-radialmenu .rseg.inner { fill: rgba(15,20,35,.94); }",
    ".kd-radialmenu .rseg.empty { fill: rgba(18,22,36,.5); stroke: #262d44; }",
    // 指を倒している扇。**白く抜いて**、いまどれが選ばれているかを一目で分かるようにする
    ".kd-radialmenu .rseg.on { fill: #ffffff; stroke: #ffffff; }",
    ".kd-radialmenu .rhub { fill: rgba(12,16,28,.96); stroke: #39425f; stroke-width: 1; }",
    ".kd-radialmenu .rhub.on { stroke: #ffffff; stroke-width: 2; }",
    ".kd-radialmenu .rname { position: absolute; transform: translate(-50%, -50%);",
    "  text-align: center; font-size: 12px; font-weight: 600; line-height: 1.15;",
    "  color: #e8edff; white-space: pre-line; }",
    ".kd-radialmenu .rname.inner { font-size: 11px; color: #b9c3de; }",
    ".kd-radialmenu .rname.empty { color: #59627d; }",
    ".kd-radialmenu .rname.on { color: #101526; font-weight: 700; }",
    ".kd-radialmenu .rhub-label { position: absolute; transform: translate(-50%, -50%);",
    "  text-align: center; font-size: 11px; color: #97a1bd; line-height: 1.2; }",
    // 指がメニューを隠すので、決まる中身は必ず**上**（上に余白が無ければ下）へ大きく出す
    ".kd-radialmenu .rtip { position: absolute; transform: translate(-50%, -50%);",
    "  max-width: 86vw; padding: 6px 12px; border-radius: 999px; text-align: center;",
    "  background: rgba(8,11,20,.95); border: 1px solid #39425f; color: #ffffff;",
    "  font-size: 15px; font-weight: 700; white-space: nowrap; overflow: hidden;",
    "  text-overflow: ellipsis; box-shadow: 0 6px 18px rgba(0,0,0,.55); }",
    ".kd-radialmenu .rtip.cancel { color: #b7bfd6; font-weight: 600; }",

    // ---- キーボード ----
    ".kd-surface .kbgrid { display: grid; grid-auto-rows: minmax(0, 1fr); gap: var(--kd-key-gap); height: 100%; }",
    ".kd-surface .kbgrid.tiny .key { font-size: 22px; }",
    ".kd-surface .key {",
    "  min-width: 0; min-height: 0; border: 0; border-radius: var(--kd-key-radius);",
    "  background: var(--kd-panel); color: var(--kd-ink); font: inherit; font-size: 15px;",
    "  display: flex; flex-direction: column; align-items: center; justify-content: center;",
    "  line-height: 1.1; touch-action: manipulation; user-select: none; padding: 2px;",
    "  overflow: hidden; white-space: nowrap; text-overflow: ellipsis;",
    "  position: relative;",
    "}",
    ".kd-surface .key small { font-size: 10px; color: var(--kd-ink-muted); }",
    ".kd-surface .key:not(.dual) small { position: absolute; top: 3px; right: 6px; line-height: 1; }",
    ".kd-surface .key.pressed { background: var(--kd-accent); }",
    '.kd-surface .key[data-empty="true"] { background: var(--kd-empty); color: var(--kd-ink-dim); }',
    // レイヤーキーを常時色分けしない。**いま効いているかどうか**は
    // 盤面全体の色で示す（下の .layer-on）。キーそのものは他と同じ色にする。
    // 常時青いと「押していないのに効いているように見える」ため。
    ".kd-surface .key.fn { background: var(--kd-panel); }",

    // 押している間だけレイヤーが効く（mo）。効いている間は盤面全体を青く染める。
    // どのキーを押したかではなく「いま盤面が別モードだ」を伝えるのが目的。
    ".kd-surface .kbgrid.layer-on { background: rgba(45,103,217,.20); border-radius: 10px; }",
    ".kd-surface .kbgrid.layer-on .key { background: var(--kd-accent-soft); }",
    ".kd-surface .kbgrid.layer-on .key[data-empty=\"true\"] { background: var(--kd-empty); }",
    ".kd-surface .key.holding { background: var(--kd-ok); color: #06210f; transform: scale(.94); }",

    // ---- Deck ----
    ".kd-surface .deckgrid { display: grid; gap: 6px; width: 100%; margin: 0 auto; flex: 0 0 auto; }",
    ".kd-surface .deckgrid.render-list { grid-template-columns: 1fr !important; gap: 5px; }",
    ".kd-surface .deckgrid.render-list .slot {",
    "  aspect-ratio: auto; min-height: 42px; flex-direction: row;",
    "  justify-content: flex-start; gap: 8px; padding: 6px 10px; text-align: left;",
    "}",
    ".kd-surface .deckgrid.render-list .slot .lb {",
    "  font-size: 13px; color: var(--kd-ink); flex: 1 1 auto; min-width: 0;",
    "  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
    "}",
    ".kd-surface .slot {",
    "  aspect-ratio: 1 / 1; width: 100%; min-width: 0;",
    "  border: 1px solid var(--kd-line); border-radius: var(--kd-slot-radius);",
    "  background: var(--kd-panel); color: var(--kd-ink); font: inherit;",
    "  display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 3px;",
    "  padding: 3px; overflow: hidden; text-align: center;",
    "  touch-action: manipulation; user-select: none; position: relative;",
    "}",
    ".kd-surface .slot .ico { font-size: 18px; line-height: 1; }",
    // アイコン画像のあるボタンだけ、画像をボタンの残りいっぱいに**収める**（ラベルは下に残す）。
    // 大きさを指定しないと画像が元の寸法（256px 等）のまま出て、はみ出した分が切られ、
    // 真ん中だけが拡大されて見えていた（2026-09-22、アイコン付きボタンを初めて置いて判明）。
    ".kd-surface .slot.has-icon .ico { flex: 1 1 0; min-height: 0; width: 100%;",
    "  display: flex; align-items: center; justify-content: center; }",
    ".kd-surface .slot.has-icon .ico img { display: block; max-width: 100%; max-height: 100%;",
    "  object-fit: contain; }",
    ".kd-surface .slot .lb { font-size: 10px; line-height: 1.15; color: var(--kd-ink-muted); overflow: hidden; }",
    // V2.1: 地色を付けたタイル。色は slot.color（#rrggbb）から来る。
    // 色の上では、細い灰色の文字は読めない。**文字と縁取りを色の側に合わせる**
    ".kd-surface .slot.tinted { border-color: rgba(255,255,255,.22); }",
    ".kd-surface .slot.tinted .lb { color: rgba(255,255,255,.92); font-weight: 600;",
    "  text-shadow: 0 1px 2px rgba(0,0,0,.55); }",
    ".kd-surface .slot.tinted .ico { color: #ffffff; }",
    ".kd-surface .slot.pressed { background: var(--kd-accent); border-color: var(--kd-accent); }",
    '.kd-surface .slot[data-empty="true"] { background: var(--kd-empty); border-style: dashed; border-color: #232b48; }',
    '.kd-surface .slot[data-empty="true"] .lb { color: var(--kd-ink-dim); }',

    // ---- 押した感（interactive のときだけ意味がある） ----
    ".kd-surface .key, .kd-surface .slot {",
    "  transition: transform 140ms cubic-bezier(.2, .9, .3, 1.3),",
    "              background-color 140ms ease-out, filter 140ms ease-out;",
    "  will-change: transform;",
    "}",
    ".kd-surface .key.pressing, .kd-surface .slot.pressing {",
    "  transition: none; transform: scale(.94); filter: brightness(1.35);",
    "}",
    ".kd-surface .key:active, .kd-surface .slot:active { background: var(--kd-accent); }",

    // ---- プレビュー（押せない）----
    // 押下の見た目（:active / hover）を殺す。触っても何も起きないことを形でも示す。
    ".kd-preview, .kd-preview * { pointer-events: none !important; }",
    ".kd-preview .key, .kd-preview .slot { cursor: default; }",
    ".kd-preview .key:active, .kd-preview .slot:active { background: var(--kd-panel); }",
    // 区画が小さいときは文字が潰れて読めないので、形だけ見せる
    ".kd-preview.tiny-labels .key, .kd-preview.tiny-labels .slot .lb { font-size: 0; }",
    ".kd-preview.tiny-labels .key small { display: none; }",

    ".kd-surface .oops { color: var(--kd-ink-dim); font-size: 12px; padding: 8px; }",
    "@media (prefers-reduced-motion: reduce) {",
    "  .kd-surface .key, .kd-surface .slot { transition: background-color 100ms ease-out; }",
    "  .kd-surface .key.pressing, .kd-surface .slot.pressing { transform: none; }",
    "}",
  ].join("\n");

  /// CSSを1度だけ挿す。何度呼んでも増えない。
  function injectStyles(doc) {
    var d = doc || document;
    if (d.getElementById(STYLE_ID)) return;
    var style = d.createElement("style");
    style.id = STYLE_ID;
    style.textContent = CSS;
    d.head.appendChild(style);
  }

  /// D3と同じ規則でラベルを引く（有効レイヤーの番号が大きい順・transは素通り）。
  /// **表示専用**。発火の判定はHub側にしかない。
  function resolveDisplay(keymap, state, keyId) {
    var s = state || {};
    var active = new Set([0].concat(s.momentary || [], s.toggled || []));
    var ordered = Array.from(active).sort(function (a, b) { return b - a; });
    for (var i = 0; i < ordered.length; i += 1) {
      var layer = keymap.layers.find(function (l) { return l.id === ordered[i]; });
      if (!layer) continue;
      var def = layer.keys[keyId];
      if (def && def.action.t !== "trans") return def;
    }
    return null;
  }

  /// 1行目を「右上の小さな印」として出すか、「上下2段」にするかを決める。
  ///
  /// 右上の印にするのは**数字ヒント**（vol1.3のQ〜Pの 1〜0）と
  /// **記号1文字**（📁 のような目印）だけ。どちらも主役ではなく添え物で、
  /// 2段にすると本文と同じ重さになって読みにくい。
  /// 「日本語/英数」「10秒/戻る」のような**言葉の2段**はそのまま2段で出す。
  function isCornerBadge(line) {
    if (/^[0-9]+$/.test(line)) return true;
    return Array.from(line).length === 1 && !/[\p{L}\p{N}]/u.test(line);
  }

  function renderLabel(btn, label) {
    btn.textContent = "";
    var lines = String(label == null ? "" : label).split("\n");
    if (lines.length >= 2) {
      var small = btn.ownerDocument.createElement("small");
      small.textContent = lines[0];
      btn.appendChild(small);
      btn.appendChild(btn.ownerDocument.createTextNode(lines.slice(1).join("\n")));
      if (!isCornerBadge(lines[0])) btn.classList.add("dual");
    } else {
      btn.textContent = lines[0];
    }
  }

  function oops(host, message) {
    var div = host.ownerDocument.createElement("div");
    div.className = "oops";
    div.textContent = message;
    host.appendChild(div);
  }

  /// キーボードを描く。
  /// `keymap` は board を持つもの、`state` は {momentary, toggled}。
  /// options: { interactive, onDown(keyId, def), onUp(keyId, def), press(el, ev), release(el) }
  // ------------------------------------------------------------------
  // ダイヤル（jog）。
  //
  // **これはキーが2つしかないキーボードでしかない。** 目盛りを1つ越えるたびに
  // CW（時計回り）か CCW（反時計回り）を1回押す。何が起きるかはキーマップが
  // 決めるので、コマ送りにも音量にも化ける（不変条件1: 送るのは位置IDだけ）。
  //
  // 角度は**差分**を足していく。絶対角度で判定すると、つまみのどこを掴んだかで
  // 手ごたえが変わってしまう。0°/360°をまたぐ飛びは -180〜180 に畳んで防ぐ。
  // ------------------------------------------------------------------
  // ------------------------------------------------------------------
  // 目盛り音。
  //
  // **なぜ <audio> ではなく Web Audio か**: 目盛りは1秒に何度も鳴る。
  // <audio> は再生中に同じ要素をもう一度鳴らせず、毎回作り直すと詰まる。
  // Web Audio なら1つの音を何重にも重ねて鳴らせる。
  //
  // **iPadで鳴るか**: 音を出す仕掛けは「指が触れた瞬間」に作らないと
  // iOSが止める。そのため最初の pointerdown で用意する。
  // ただし**本体横の消音スイッチが入っていると鳴らない**（ブラウザからは
  // 解除できない）。鳴らないときはまずそこを疑うこと。
  // ------------------------------------------------------------------
  var Click = {
    ctx: null, buffer: null, failed: false,
    /// 指が触れた瞬間に呼ぶ。2回目以降は何もしない。
    arm: function () {
      if (this.ctx || this.failed) return;
      var Ctor = global.AudioContext || global.webkitAudioContext;
      if (!Ctor) { this.failed = true; return; }
      var self = this;
      try {
        this.ctx = new Ctor();
        // iOSは触った瞬間に resume しないと止まったままになる
        if (this.ctx.state === "suspended") this.ctx.resume();
        global.fetch("/sounds/detent.mp3")
          .then(function (r) { return r.arrayBuffer(); })
          .then(function (b) { return self.ctx.decodeAudioData(b); })
          .then(function (buf) { self.buffer = buf; })
          .catch(function (error) {
            self.failed = true;
            console.error("[KD][ERR][SOUND] 目盛り音を読み込めません", error);
          });
      } catch (error) {
        this.failed = true;
        console.error("[KD][ERR][SOUND] 音を用意できません", error);
      }
    },
    /// 先頭の一瞬だけを鳴らす。
    /// 用意した音は全体で1.08秒あるが、実際に音が出ているのは**先頭40msだけ**で
    /// 残りは無音。全部を再生すると、目盛りのたびに1秒生き続ける無音の再生が
    /// 積み上がるので、頭だけを切り出して鳴らす。
    play: function (gain) {
      if (!this.ctx || !this.buffer) return;
      var src = this.ctx.createBufferSource();
      src.buffer = this.buffer;
      var vol = this.ctx.createGain();
      vol.gain.value = gain == null ? 0.5 : gain;
      src.connect(vol);
      vol.connect(this.ctx.destination);
      src.start(0, 0, Math.min(0.06, this.buffer.duration));
    },
  };

  function renderJog(host, keymap, state, options) {
    var opt = options || {};
    injectStyles(host.ownerDocument);
    host.classList.add("kd-surface");
    if (!opt.interactive && !opt.editable) host.classList.add("kd-preview");

    if (!keymap || !keymap.jog || !keymap.board) {
      oops(host, "ダイヤルが見つかりません");
      return null;
    }
    // 形が wheel なら縦長のホイールを描く。送るもの（CW/CCW）は丸いダイヤルと同じ
    if (keymap.jog.shape === "wheel") return renderWheel(host, keymap, state, opt);

    var doc = host.ownerDocument;
    var NS = "http://www.w3.org/2000/svg";
    var DETENT = keymap.jog.detentDeg || 15;
    var C = 90, R_TICK = 76, R_PROG = 70, R_KNOB = 55;

    function el(tag, attrs) {
      var e = doc.createElementNS(NS, tag);
      for (var k in attrs) if (Object.prototype.hasOwnProperty.call(attrs, k)) e.setAttribute(k, attrs[k]);
      return e;
    }
    function polar(r, deg) {
      var a = deg * Math.PI / 180;
      return [C + r * Math.sin(a), C - r * Math.cos(a)];
    }

    var wrap = doc.createElement("div");
    wrap.className = "jogwrap";
    var svg = el("svg", { class: "jogsvg", viewBox: "0 0 180 180" });

    var uid = "jog-" + (keymap.keymapId || "x").replace(/[^a-z0-9_]/gi, "");
    var defs = el("defs");
    var grad = el("radialGradient", { id: uid + "-k", cx: "38%", cy: "30%", r: "75%" });
    grad.appendChild(el("stop", { offset: "0%", "stop-color": "#6a7184" }));
    grad.appendChild(el("stop", { offset: "55%", "stop-color": "#2a3040" }));
    grad.appendChild(el("stop", { offset: "100%", "stop-color": "#141824" }));
    defs.appendChild(grad);
    svg.appendChild(defs);

    svg.appendChild(el("circle", { cx: C, cy: C, r: 86, fill: "#0b0e18" }));
    // 目盛りは detentDeg のとおりに刻む。見た目と手ごたえを必ず一致させる
    // （見た目が15°刻みなのに30°で1段だと、回しても進まない不良品に見える）。
    var count = Math.max(1, Math.round(360 / DETENT));
    for (var i = 0; i < count; i++) {
      var deg = i * (360 / count);
      var a = polar(R_TICK, deg), b = polar(i % 3 === 0 ? 84 : 81, deg);
      svg.appendChild(el("line", {
        class: "jogtick" + (i % 3 === 0 ? " major" : ""),
        x1: a[0], y1: a[1], x2: b[0], y2: b[1],
      }));
    }
    // 進み具合の弧。1周で一巡する値（音量など）にだけ意味があるので、
    // コマ送りのように終わりの無いダイヤルでは出さない。
    var CIRC = 2 * Math.PI * R_PROG;
    var prog = null;
    if (keymap.jog.ring !== false) {
      prog = el("circle", {
        class: "jogprog", cx: C, cy: C, r: R_PROG,
        "stroke-dasharray": CIRC, "stroke-dashoffset": CIRC,
        transform: "rotate(-90 " + C + " " + C + ")",
      });
      svg.appendChild(prog);
    }
    var knob = el("g");
    knob.appendChild(el("circle", { cx: C, cy: C, r: 60, fill: "#1a1f2e" }));
    knob.appendChild(el("circle", { cx: C, cy: C, r: R_KNOB, fill: "url(#" + uid + "-k)" }));
    knob.appendChild(el("circle", { cx: C, cy: C - 40, r: 5, fill: "var(--kd-accent)" }));
    svg.appendChild(knob);
    wrap.appendChild(svg);

    // 回すと何が起きるかを添える。ダイヤルは見ただけでは用途が分からない
    var labels = doc.createElement("div");
    labels.className = "joglabels";
    var ccwDef = resolveDisplay(keymap, state, "CCW");
    var cwDef = resolveDisplay(keymap, state, "CW");
    var left = doc.createElement("span");
    left.textContent = "◀ " + ((ccwDef && ccwDef.label) || "CCW").split("\n")[0];
    var right = doc.createElement("b");
    right.textContent = ((cwDef && cwDef.label) || "CW").split("\n")[0] + " ▶";
    labels.appendChild(left);
    labels.appendChild(right);
    wrap.appendChild(labels);
    host.appendChild(wrap);

    if (!opt.interactive) return wrap;

    // ---- 手ごたえ ----------------------------------------------------
    // 指の角度（want）と、つまみの角度（total）を**分けて持つ**。
    // つまみは want へ向かって少しずつ近づくだけなので、重いほど遅れて
    // 付いてくる。**目盛りは「つまみ」の角度で数える**ので、指を速く回しても
    // 段は飛ばず、1段ずつ順に鳴る。指を離したあとも、残りを回り切ってから止まる。
    //
    // 勢い（慣性）は入れていない。行き過ぎるとコマ送りが1コマ余分に進み、
    // 狙った画で止められなくなるため。
    var dragging = false, lastAng = 0;
    var want = 0;      // 指が示している角度（累積）
    var total = 0;     // つまみの角度（累積）。目盛りはこちらで数える
    var carry = 0;
    var EPS = 1e-6;
    var chase = 1 - Math.min(95, Math.max(0, keymap.jog.weight || 0)) / 100;
    var frame = null;

    function angleAt(event) {
      var r = svg.getBoundingClientRect();
      var dx = event.clientX - (r.left + r.width / 2);
      var dy = event.clientY - (r.top + r.height / 2);
      return Math.atan2(dx, -dy) * 180 / Math.PI;
    }
    function draw() {
      knob.setAttribute("transform", "rotate(" + total + " " + C + " " + C + ")");
      if (prog) {
        var frac = (((total % 360) + 360) % 360) / 360;
        prog.setAttribute("stroke-dashoffset", CIRC * (1 - frac));
      }
    }
    /// つまみを want へ少し近づけ、越えた目盛りぶんだけ鳴らす。
    function step() {
      frame = null;
      var diff = want - total;
      var move = diff * chase;
      // 近づき切ったら終わり。止め時を決めないと永久に微動し続ける
      if (Math.abs(diff) < 0.01) { move = diff; }
      total += move;
      carry += move;
      var fired = 0;
      while (carry >= DETENT - EPS) { carry -= DETENT; fired += 1; if (opt.onDetent) opt.onDetent("CW"); }
      while (carry <= -DETENT + EPS) { carry += DETENT; fired += 1; if (opt.onDetent) opt.onDetent("CCW"); }
      if (fired > 0 && keymap.jog.sound) {
        // 一度に何段も越えたときは、うるさくならないよう1回だけ鳴らす
        Click.play(0.5);
      }
      draw();
      if (Math.abs(want - total) >= 0.01) schedule();
    }
    function schedule() {
      if (frame === null) frame = global.requestAnimationFrame(step);
    }
    /// 指の動きを want に足す。**離した瞬間の位置も必ず通す**（最後の動きを取りこぼさない）。
    function applyMove(event) {
      var a = angleAt(event);
      var d = a - lastAng;
      if (d > 180) d -= 360;
      if (d < -180) d += 360;
      lastAng = a;
      want += d;
      schedule();
    }

    svg.addEventListener("pointerdown", function (event) {
      event.preventDefault();
      dragging = true;
      lastAng = angleAt(event);
      if (keymap.jog.sound) Click.arm();   // 触れた瞬間でないとiOSが音を止める
      try { svg.setPointerCapture(event.pointerId); } catch (e) { /* 未対応環境 */ }
    });
    svg.addEventListener("pointermove", function (event) {
      if (dragging) applyMove(event);
    });
    svg.addEventListener("pointerup", function (event) {
      if (!dragging) return;
      applyMove(event);
      dragging = false;
    });
    svg.addEventListener("pointercancel", function () { dragging = false; });
    draw();
    return wrap;
  }

  /// 縦長のホイール（jog の shape=wheel）。上下にこすると、detentPx 動くごとに
  /// 1段送る。指を下へ＝CW、上へ＝CCW（丸いダイヤルと同じ2キー）。
  /// 溝は指にぴったり付いて動く（重みは付けない）。細い区画では遅れて付いてくると
  /// どこまで回したか分からなくなるため。
  function renderWheel(host, keymap, state, opt) {
    var doc = host.ownerDocument;
    var STEP = keymap.jog.detentPx || 18;

    var wrap = doc.createElement("div");
    wrap.className = "wheelwrap";
    var up = resolveDisplay(keymap, state, "CCW");
    var down = resolveDisplay(keymap, state, "CW");
    var top = doc.createElement("div");
    top.className = "wheellb";
    top.textContent = "▲ " + ((up && up.label) || "CCW").split("\n")[0];
    var drum = doc.createElement("div");
    drum.className = "wheeldrum";
    var bottom = doc.createElement("div");
    bottom.className = "wheellb";
    bottom.textContent = ((down && down.label) || "CW").split("\n")[0] + " ▼";
    wrap.appendChild(top);
    wrap.appendChild(drum);
    wrap.appendChild(bottom);
    host.appendChild(wrap);

    if (!opt.interactive) return wrap;

    var dragging = false, lastY = 0, offset = 0, carry = 0;
    function applyMove(event) {
      var dy = event.clientY - lastY;
      lastY = event.clientY;
      offset += dy;
      carry += dy;
      drum.style.backgroundPositionY = offset + "px";
      var fired = 0;
      while (carry >= STEP) { carry -= STEP; fired += 1; if (opt.onDetent) opt.onDetent("CW"); }
      while (carry <= -STEP) { carry += STEP; fired += 1; if (opt.onDetent) opt.onDetent("CCW"); }
      // 一度に何段も越えたときは1回だけ鳴らす（ダイヤルと同じ）
      if (fired > 0 && keymap.jog.sound) Click.play(0.5);
    }
    drum.addEventListener("pointerdown", function (event) {
      event.preventDefault();
      dragging = true;
      lastY = event.clientY;
      carry = 0;   // 前回の端数を持ち越さない。触り直した直後に1段飛ぶのを防ぐ
      drum.classList.add("on");
      if (keymap.jog.sound) Click.arm();   // 触れた瞬間でないとiOSが音を止める
      try { drum.setPointerCapture(event.pointerId); } catch (e) { /* 未対応環境 */ }
    });
    drum.addEventListener("pointermove", function (event) {
      if (dragging) applyMove(event);
    });
    function end(event, useLast) {
      if (!dragging) return;
      if (useLast) applyMove(event);
      dragging = false;
      drum.classList.remove("on");
    }
    drum.addEventListener("pointerup", function (event) { end(event, true); });
    drum.addEventListener("pointercancel", function (event) { end(event, false); });
    return wrap;
  }

  /// 扇の方向id。上から時計回りに等間隔。**Rust側の RADIAL_DIRS_* と同じ並び**で、
  /// ここを変えると盤面が要求するキーidが合わなくなる（proto-keymap/src/lib.rs）。
  var RADIAL_DIRS = {
    4: ["N", "E", "S", "W"],
    6: ["N", "NE", "SE", "S", "SW", "NW"],
    8: ["N", "NE", "E", "SE", "S", "SW", "W", "NW"],
  };

  var SVG_NS = "http://www.w3.org/2000/svg";

  function svgEl(doc, name, attrs) {
    var el = doc.createElementNS(SVG_NS, name);
    Object.keys(attrs || {}).forEach(function (k) { el.setAttribute(k, attrs[k]); });
    return el;
  }

  /// 上を0°、時計回りに測った角度の点。
  function polarPt(cx, cy, r, deg) {
    var a = (deg - 90) * Math.PI / 180;
    return [cx + r * Math.cos(a), cy + r * Math.sin(a)];
  }

  /// 輪の一部（扇形）のパス。r0=内側の半径、r1=外側の半径。
  function ringSector(cx, cy, r0, r1, a0, a1) {
    var big = (a1 - a0) > 180 ? 1 : 0;
    var o0 = polarPt(cx, cy, r1, a0), o1 = polarPt(cx, cy, r1, a1);
    var i1 = polarPt(cx, cy, r0, a1), i0 = polarPt(cx, cy, r0, a0);
    return "M " + o0[0] + " " + o0[1]
      + " A " + r1 + " " + r1 + " 0 " + big + " 1 " + o1[0] + " " + o1[1]
      + " L " + i1[0] + " " + i1[1]
      + " A " + r0 + " " + r0 + " 0 " + big + " 0 " + i0[0] + " " + i0[1]
      + " Z";
  }

  /// 1マスの**二層ラジアルメニュー**（階層型パイメニュー）。
  ///
  /// 押すと指の位置を中心に扇形の選択肢が開く。決め方は2つの物差しだけ:
  ///   角度 … どの扇か（radial.sectors が 4 / 6 / 8）
  ///   距離 … 中心=取り消し ／ 内側の輪=細かい操作 ／ 外側の輪=大分類（radial.rings が 2 のとき）
  /// 指を離すと白く光っている扇が決まり、その方向のキーを1回押す（down→up）。
  /// 中心で離す・指が取られる（pointercancel）と何も送らない。
  ///
  /// 内側の輪のキーidは方向のうしろに `2` を付けたもの（`N` の内側は `N2`）。
  /// **端末が送るのは位置idだけ**で、何が起きるかは layer の action が決める（不変条件1）。
  ///
  /// 判定の中心は「指を置いた点」ではなく「メニューを描いた中心」にする。
  /// 距離で輪を選ぶ以上、見えている絵と判定がずれると、どこで離せばよいか分からなくなるため。
  function renderRadial(host, keymap, state, opt) {
    var doc = host.ownerDocument;
    var cfg = keymap.radial || {};
    var dirs = RADIAL_DIRS[cfg.sectors] || RADIAL_DIRS[4];
    var rings = cfg.rings === 2 ? 2 : 1;
    var STEP = 360 / dirs.length;
    var GAP = 1.4;     // 扇どうしの隙間（度）

    // 半径（px）。指で狙える 44px の幅を輪ごとに確保する
    var HUB = 30;                                  // 真ん中の丸（取り消し）
    var BAND0 = rings === 2 ? [34, 100] : [34, 110];  // 内側の輪（2層のときだけ中身が変わる）
    var BAND1 = [104, 158];                        // 外側の輪
    var REACH = rings === 2 ? BAND1[1] : BAND0[1];

    // 各扇の中身。ring 0 = 内側（id に 2 が付く）、ring 1 = 外側（idそのまま）
    var SLOTS = [];
    dirs.forEach(function (dir, i) {
      if (rings === 2) SLOTS.push({ keyId: dir + "2", dir: dir, i: i, ring: 0, band: BAND0 });
      SLOTS.push({ keyId: dir, dir: dir, i: i, ring: rings === 2 ? 1 : 0, band: rings === 2 ? BAND1 : BAND0 });
    });

    var btn = doc.createElement("button");
    btn.type = "button";
    btn.className = "radialbtn";
    var lb = doc.createElement("span");
    lb.className = "rlabel";
    lb.textContent = cfg.label || "◎";
    btn.appendChild(lb);
    [{ mark: "▲", cls: "n" }, { mark: "▶", cls: "e" }, { mark: "▼", cls: "s" }, { mark: "◀", cls: "w" }]
      .forEach(function (d) {
        var a = doc.createElement("span");
        a.className = "rarrow " + d.cls;
        a.textContent = d.mark;
        btn.appendChild(a);
      });
    host.appendChild(btn);
    if (!opt.interactive) { btn.disabled = true; return btn; }

    var menu = null, tip = null, hub = null, active = false;
    var segs = [], names = [], defs = [];
    var cx = 0, cy = 0, scale = 1, picked = -1;
    var startX = 0, startY = 0, moved = false;
    var GRACE = 12;   // これだけ動くまでは何も選ばない（ただ押しただけで暴発させない）

    function labelOf(def, keyId) {
      if (!def || def.action.t === "none") return "—";
      return def.label || keyId;
    }

    function open(event) {
      var win = doc.defaultView;
      // 画面が扇より狭いときは全体を縮める。端で押されたら、中心だけ内側へ寄せる
      scale = Math.min(1, (win.innerWidth - 10) / (REACH * 2), (win.innerHeight - 10) / (REACH * 2));
      var reach = REACH * scale;
      cx = Math.min(Math.max(event.clientX, reach + 4), win.innerWidth - reach - 4);
      cy = Math.min(Math.max(event.clientY, reach + 4), win.innerHeight - reach - 4);

      menu = doc.createElement("div");
      menu.className = "kd-radialmenu";

      var size = reach * 2 + 8, half = size / 2;
      var svg = svgEl(doc, "svg", { class: "rsvg", width: size, height: size, viewBox: "0 0 " + size + " " + size });
      svg.style.left = (cx - half) + "px";
      svg.style.top = (cy - half) + "px";
      // 扇を先に入れる。名前はこのあとに足して、扇の**上**に載せる
      menu.appendChild(svg);

      segs = []; names = []; defs = [];
      SLOTS.forEach(function (slot) {
        var def = resolveDisplay(keymap, state, slot.keyId);
        var empty = !def || def.action.t === "none";
        var mid = slot.i * STEP;
        var path = svgEl(doc, "path", {
          class: "rseg" + (slot.ring === 0 && rings === 2 ? " inner" : "") + (empty ? " empty" : ""),
          d: ringSector(half, half, slot.band[0] * scale, slot.band[1] * scale,
            mid - STEP / 2 + GAP, mid + STEP / 2 - GAP),
        });
        svg.appendChild(path);

        // 名前を置く幅は、扇の「厚み」と「弧の幅」の広いほう。
        // 厚みだけに合わせると内側の輪で縦長になり、弧だけに合わせると外側で切れる
        var rMid = (slot.band[0] + slot.band[1]) / 2 * scale;
        var arcW = 2 * rMid * Math.sin(STEP * Math.PI / 360) * 0.86;
        var pt = polarPt(cx, cy, rMid, mid);
        var nm = doc.createElement("div");
        nm.className = "rname" + (slot.ring === 0 && rings === 2 ? " inner" : "") + (empty ? " empty" : "");
        nm.textContent = labelOf(def, slot.keyId);
        nm.style.left = pt[0] + "px";
        nm.style.top = pt[1] + "px";
        nm.style.maxWidth = Math.max((slot.band[1] - slot.band[0]) * scale - 6, arcW) + "px";

        segs.push(path); names.push(nm); defs.push(empty ? null : def);
        menu.appendChild(nm);
      });

      hub = svgEl(doc, "circle", { class: "rhub on", cx: half, cy: half, r: HUB * scale });
      svg.appendChild(hub);

      var hubLabel = doc.createElement("div");
      hubLabel.className = "rhub-label";
      hubLabel.textContent = "取消";
      hubLabel.style.left = cx + "px";
      hubLabel.style.top = cy + "px";
      menu.appendChild(hubLabel);

      // 指はメニューを隠すので、決まる中身は上に出す。上に余白が無ければ下へ
      tip = doc.createElement("div");
      tip.className = "rtip cancel";
      tip.textContent = "取消";
      var above = cy - reach - 22;
      tip.style.left = Math.min(Math.max(cx, 90), win.innerWidth - 90) + "px";
      tip.style.top = (above > 20 ? above : cy + reach + 22) + "px";
      menu.appendChild(tip);

      doc.body.appendChild(menu);
    }

    function close() {
      if (menu && menu.parentNode) menu.parentNode.removeChild(menu);
      menu = null; tip = null; hub = null; segs = []; names = []; defs = [];
      btn.classList.remove("on");
      active = false;
    }

    /// いま指が指しているスロットの番号。中心付近なら -1（取り消し）。
    ///
    /// 画面の端のマスに置くと、メニューは全体が見えるよう内側へ寄って開く。
    /// つまり**押した指は最初からどれかの扇の上に乗っている**。そのまま決めてしまうと
    /// ただ触っただけで暴発するので、GRACE だけ動かすまでは何も選ばない。
    /// 動かす向きは常にメニューの中心側＝画面の内側なので、端のマスでも全部に届く。
    function hit(event) {
      if (!moved) {
        if (Math.hypot(event.clientX - startX, event.clientY - startY) < GRACE) return -1;
        moved = true;
      }
      var dx = event.clientX - cx, dy = event.clientY - cy;
      var dist = Math.hypot(dx, dy);
      if (dist < HUB * scale) return -1;
      var ang = Math.atan2(dx, -dy) * 180 / Math.PI;
      if (ang < 0) ang += 360;
      var i = Math.round(ang / STEP) % dirs.length;
      // 外へ出しすぎても外側の輪のまま（「外周外は最後の扇を保つ」）
      var ring = rings === 2 && dist >= BAND1[0] * scale ? 1 : 0;
      for (var n = 0; n < SLOTS.length; n += 1) {
        if (SLOTS[n].i === i && SLOTS[n].ring === ring) return n;
      }
      return -1;
    }

    function paint(idx) {
      picked = idx;
      segs.forEach(function (s, i) { s.classList.toggle("on", i === idx); });
      names.forEach(function (n, i) { n.classList.toggle("on", i === idx); });
      hub.classList.toggle("on", idx === -1);
      var cancel = idx === -1;
      tip.classList.toggle("cancel", cancel || !defs[idx]);
      tip.textContent = cancel ? "取消"
        : (defs[idx] ? (defs[idx].label || SLOTS[idx].keyId).replace(/\n/g, " ") : "空き");
    }

    btn.addEventListener("pointerdown", function (event) {
      event.preventDefault();
      if (active) return;
      active = true;
      picked = -1;
      startX = event.clientX; startY = event.clientY; moved = false;
      btn.classList.add("on");
      try { btn.setPointerCapture(event.pointerId); } catch (e) { /* 未対応環境 */ }
      open(event);
      paint(-1);
    });
    btn.addEventListener("pointermove", function (event) {
      if (!active) return;
      var idx = hit(event);
      if (idx !== picked) paint(idx);
    });
    btn.addEventListener("pointerup", function (event) {
      if (!active) return;
      var idx = hit(event);
      // 中身が無い扇は、光らせても何も送らない（Hub側で空振りさせない）
      var chosen = idx >= 0 && defs[idx] ? SLOTS[idx].keyId : null;
      close();
      if (chosen) {
        if (opt.onDown) opt.onDown(chosen);
        if (opt.onUp) opt.onUp(chosen);
      }
    });
    btn.addEventListener("pointercancel", close);
    return btn;
  }

  function renderKeyboard(host, keymap, state, options) {
    var opt = options || {};
    injectStyles(host.ownerDocument);
    host.classList.add("kd-surface");
    if (!opt.interactive && !opt.editable) host.classList.add("kd-preview");

    if (!keymap || !keymap.board) {
      oops(host, "キーボードが見つかりません");
      return null;
    }
    // ラジアルボタン。**キー編集画面（editable）では普通の盤面のまま**にして、
    // 4方向のキー（N/E/S/W）の中身をいつもどおり差し替えられるようにする
    if (keymap.radial && !opt.editable) return renderRadial(host, keymap, state, opt);

    var doc = host.ownerDocument;
    // 一時レイヤー（mo）が効いているか。効いている間だけ盤面を染める。
    // toggle（tg）は含めない。押しっぱなしでない状態まで染めると、
    // 「戻し忘れ」と「押している最中」が見分けられなくなる。
    var momentaryOn = !!(state && state.momentary && state.momentary.length > 0);

    var grid = doc.createElement("div");
    grid.className = "kbgrid"
      + (keymap.board.cols <= 4 ? " tiny" : "")
      + (momentaryOn ? " layer-on" : "");
    grid.dataset.keymapId = keymap.keymapId || "";
    grid.style.gridTemplateColumns = "repeat(" + keymap.board.cols + ", minmax(0, 1fr))";

    keymap.board.keys.forEach(function (boardKey) {
      var def = resolveDisplay(keymap, state, boardKey.id);
      var btn = doc.createElement("button");
      btn.className = "key";
      btn.type = "button";
      btn.dataset.keyId = boardKey.id;
      var colSpan = boardKey.colSpan || 1;
      var rowSpan = boardKey.rowSpan || 1;
      btn.style.gridColumn = boardKey.col + " / " + (boardKey.col + colSpan);
      btn.style.gridRow = boardKey.row + " / " + (boardKey.row + rowSpan);

      var isEmpty = !def || def.action.t === "none";
      if (isEmpty) {
        btn.dataset.empty = "true";
        renderLabel(btn, "");
      } else {
        if (def.action.t === "mo" || def.action.t === "tg") btn.classList.add("fn");
        renderLabel(btn, def.label);
        if (opt.interactive) bindKey(btn, boardKey.id, def, opt);
      }
      grid.appendChild(btn);
    });

    host.appendChild(grid);
    return grid;
  }

  function bindKey(btn, keyId, def, opt) {
    var isHold = def.action.t === "key.hold";
    var down = function (event) {
      event.preventDefault();
      if (opt.press) opt.press(btn, event);
      if (isHold) btn.classList.add("holding");
      if (opt.onDown) opt.onDown(keyId, def);
    };
    var up = function (event) {
      event.preventDefault();
      if (opt.release) opt.release(btn);
      // 押しっぱなし中に指が外れた・取り消された場合も必ずupを送る。
      // 送らないとHub側が押しっぱなしのままになる。
      if (isHold && !btn.classList.contains("holding")) return;
      btn.classList.remove("holding");
      if (opt.onUp) opt.onUp(keyId, def);
    };
    btn.addEventListener("pointerdown", down);
    ["pointerup", "pointercancel", "pointerleave"].forEach(function (type) {
      btn.addEventListener(type, up);
    });
  }

  /// Deckを描く。options: { interactive, onPress(slotId), press, release }
  function renderDeck(host, deck, options) {
    var opt = options || {};
    injectStyles(host.ownerDocument);
    host.classList.add("kd-surface");
    if (!opt.interactive && !opt.editable) host.classList.add("kd-preview");

    if (!deck) {
      oops(host, "Deckが見つかりません");
      return null;
    }

    var doc = host.ownerDocument;
    var isList = deck.render === "list";
    var grid = doc.createElement("div");
    grid.className = "deckgrid" + (isList ? " render-list" : "");
    grid.dataset.deckId = deck.deckId || "";
    grid.style.gridTemplateColumns = "repeat(" + deck.grid.cols + ", minmax(0, 1fr))";

    var page = (deck.pages || [])[0];
    if (page) {
      page.slots.forEach(function (slot) {
        var btn = doc.createElement("button");
        btn.className = "slot";
        btn.type = "button";
        btn.dataset.slotId = slot.slotId;
        var isEmpty = slot.action.t === "none";
        if (isEmpty) btn.dataset.empty = "true";
        // V2.1: タイルの地色。Hub側が `#rrggbb` しか通さないので、ここへ直接入れてよい。
        // 空きタイルには塗らない（「置ける場所」と「置いてある物」を色で分けるため）
        if (slot.color && !isEmpty) {
          btn.style.background = slot.color;
          btn.classList.add("tinted");
        }

        var ico = doc.createElement("span");
        ico.className = "ico";
        if (slot.icon) {
          btn.classList.add("has-icon");
          var img = doc.createElement("img");
          img.src = slot.icon;
          img.alt = "";
          ico.appendChild(img);
        }
        btn.appendChild(ico);

        var lb = doc.createElement("span");
        lb.className = "lb";
        lb.textContent = slot.label;
        btn.appendChild(lb);

        btn.disabled = isEmpty || !opt.interactive;
        if (opt.interactive && !isEmpty) {
          btn.addEventListener("pointerdown", function (event) {
            event.preventDefault();
            if (opt.press) opt.press(btn, event);
            if (opt.onPress) opt.onPress(slot.slotId);
          });
          // Deckは押しっぱなしの概念が無いので、離す・外れる・取り消しの全部で戻す
          ["pointerup", "pointercancel", "pointerleave"].forEach(function (type) {
            btn.addEventListener(type, function () { if (opt.release) opt.release(btn); });
          });
        }
        grid.appendChild(btn);
      });
    }

    host.appendChild(grid);
    return grid;
  }

  /// 正方形スロットは幅で大きさが決まるため、行数が多いと区画からはみ出す。
  /// 区画の高さから1マスの上限を逆算する。
  function fitDeck(host, grid, cols, rows, minCell) {
    var gap = 6;
    var floor = minCell === undefined ? 40 : minCell;
    var style = getComputedStyle(host);
    var padY = parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
    var avail = host.clientHeight - padY;
    var cell = Math.max(floor, (avail - gap * (rows - 1)) / rows);
    grid.style.maxWidth = (cell * cols + gap * (cols - 1)) + "px";
  }

  // ================================================================
  // トークン無しで開かれたときの案内。
  //
  // トークンは起動のたびに変わる（D8）ので、URLを覚えて開く運用ができない。
  // 通常はHubが起動時にブラウザを開くが、それが失敗した場合や、
  // 古いURLをブックマークしていた場合にここへ来る。
  // **何が起きたか・どうすれば入れるか**を画面いっぱいに出す。
  // 小さなステータス行だと気づかれず「壊れている」と誤解される。
  // ================================================================
  // ------------------------------------------------------------------
  // 画面切り替え（部門 → 画面 の2階層）。
  //
  // 「PC部門」がいま実際にある画面。「ゲーム部門」は**まだ中身が無い**。
  // 将来PC画面と組み合わせ、タブレットを操作パネルにしたゲーム
  // （携帯ゲーム機の下画面のような使い方）を作るときのための
  // 場所取りで、選んでも何も起きない（disabled）。
  // ------------------------------------------------------------------
  var NAV_TREE = [
    {
      dept: "PC部門",
      items: [
        // 2026-09-30〜: トップは統合編集画面（studio.html）。旧来のレイアウト編集は /editor
        { label: "統合編集（新）", path: "/" },
        { label: "レイアウト編集（旧）", path: "/editor" },
        // 新View（2026-09-23〜）。置ける部品を探しやすく・中身が見えるように作り替えたもの。
        // 旧来の「レイアウト編集」は残してあり、どちらでも同じboardを編集・保存できる
        { label: "レイアウト編集 V2（新View）", path: "/editor_v2" },
        { label: "キー編集", path: "/keys" },
        { label: "Deck編集", path: "/deckedit" },
        { label: "接続（Gallery）", path: "/connect" },
        { label: "操作カタログ", path: "/catalog" },
        { label: "スキーマ・辞書", path: "/schema" },
        { label: "設定", path: "/settings" },
      ],
    },
    // ゲーム部門（2026-09-23〜）。道具（編集画面）はPC部門と同じものを使い、
    // ここからは**ゲーム用の中身**へ直接入る。Loupedeck の Profile と同じ考え方で、
    // 部門＝自分の持ち物への入口。画面を増やさないので、直す場所も1か所のまま
    {
      dept: "ゲーム部門",
      items: [
        { label: "盤面 iPad", path: "/?id=game_ipad" },
        { label: "盤面 iPhone 横", path: "/?id=game_iphone7_land" },
        { label: "盤面 iPhone 縦", path: "/?id=game_iphone7_port" },
        // ユーザーが自分で作った盤面（空の軌跡・iPhone 7 横 8×4）。
        // 部門は「自分の持ち物への入口」なので、こちらも並べる
        { label: "盤面 空の軌跡", path: "/?id=game_soranokiseki" },
        { label: "アクション編集", path: "/keys?keymap=game_action&layer=0" },
        { label: "移動キー編集", path: "/keys?keymap=dpad01&layer=0" },
        { label: "道具Deck編集", path: "/deckedit?deck=game" },
      ],
    },
    // 執筆部門（2026-09-28〜）。Note Story（カンバンNote）で会話劇を書くときの盤面
    // （REQ-20260928-001）。ゲーム部門と同じく、自分の持ち物への入口だけを並べる
    {
      dept: "執筆部門（Note Story）",
      items: [
        { label: "盤面 iPad", path: "/?id=note_story" },
        { label: "話者Deck編集", path: "/deckedit?deck=note_cast" },
        { label: "定型文Deck編集", path: "/deckedit?deck=note_palette" },
        { label: "書く道具キー編集", path: "/keys?keymap=note_tools&layer=0" },
      ],
    },
    // 実験（2026-09-29〜）。本番の盤面には入っていない試作。PCへは何も送らない
    {
      dept: "実験",
      items: [
        { label: "1マス多重操作の判定（REQ-20260929-001）", path: "/lab/multigesture" },
      ],
    },
    // V2.1 へ作り替える前の編集画面。**比べる・戻すためだけ**に残してある。
    // 描画も同じ日の写し（components_v1_1.js）を使うので、ここを開いても
    // いまの画面の直しは映らない
    {
      dept: "保存版（View_Ver1.1）",
      items: [
        { label: "レイアウト編集 Ver1.1", path: "/editor_v1_1" },
        { label: "キー編集 Ver1.1", path: "/keys_v1_1" },
      ],
    },
  ];

  /// host に「KeyDeck」と出ているボタンを1つ描く。押すと候補が下に出て、
  /// 選ぶと token を引き継いで画面遷移する。
  ///
  /// **表示は常に「KeyDeck」のまま**にする。ここは看板であって現在地表示ではない。
  /// いまどの画面に居るかは候補の側に ● を付けて示す。
  /// （選ぶと画面が変わって読み込み直されるので、表示は自然と「KeyDeck」へ戻る）
  /// host に「KeyDeck」と出ているボタンを1つ描く。押すと候補が上から滑り出て、
  /// 選ぶと token を引き継いで画面遷移する。
  ///
  /// **表示は常に「KeyDeck」のまま**にする。ここは看板であって現在地表示ではない。
  /// いまどの画面に居るかは候補の側に ● を付けて示す。
  ///
  /// 標準の <select> をやめて自前で組んでいる理由は2つ。
  ///   1. 開いた一覧の配色を指定できない（環境によって字が読めない組合せになる）
  ///   2. 開くときの動きを付けられない
  /// Hubが配ってきたテーマを画面へ当てる。
  /// **知らない名前は当てない**（配信が壊れたときに色が消えるより、
  /// 既定のままの方が操作を続けられる）。
  var THEMES = ["blue", "red", "green"];
  function applyTheme(name) {
    var doc = global.document;
    var root = (doc || {}).documentElement;
    if (!root) return;
    // テーマの色はこの注入スタイルの中にある。描画関数を1つも呼ばない画面
    // （トラックボール面など）でも当たるよう、ここで必ず入れておく。
    injectStyles(doc);
    if (THEMES.indexOf(name) < 0) name = "blue";
    if (name === "blue") delete root.dataset.kdTheme;
    else root.dataset.kdTheme = name;
  }

  /// Hubへ今のテーマを聞いて当てる。画面を開いた直後に1回呼ぶ。
  /// WSを持つ画面は、そのあと surface.config が来るたびに更新される。
  function syncTheme(token) {
    if (!global.fetch) return;
    global.fetch("/api/theme?token=" + encodeURIComponent(token || ""))
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (d) { if (d) applyTheme(d.theme); })
      .catch(function () { /* 取れなくても既定の見た目で動く */ });
  }

  /// 既定のものに緑の印を付ける。一覧の中で「どれが最初に出るか」を見せる。
  function markDefaultOption(sel, defaultValue) {
    for (var i = 0; i < sel.options.length; i++) {
      var opt = sel.options[i];
      var bare = opt.textContent.replace(/^✓\s*/, "");
      opt.textContent = (opt.value && opt.value === defaultValue) ? "✓ " + bare : bare;
      opt.style.color = (opt.value && opt.value === defaultValue) ? "#35c47a" : "";
    }
  }

  /// サブheader（2段目）を描く。どの画面でも同じ形・同じ場所に出す。
  ///
  ///   crumbs  … [{label, path}] ホームから始まる道筋。最後の1つが「いま居る所」
  ///   actions … [{label, id, title, primary, onClick}] その画面固有の操作。右寄せ
  ///
  /// 戻り値の `toast(text)` で、この帯の上に知らせを浮かせる。**行を占めない**の
  /// が要点で、占めると出るたびに下の中身が押し下げられる。
  function renderSubHeader(host, options) {
    injectStyles(host.ownerDocument);
    var doc = host.ownerDocument;
    var opt = options || {};
    var crumbList = opt.crumbs || [];
    var tokenPart = opt.token ? "token=" + encodeURIComponent(opt.token) : "";

    var bar = doc.createElement("div");
    bar.className = "kd-subhead";

    var crumbs = doc.createElement("nav");
    crumbs.className = "kd-crumbs";
    crumbs.setAttribute("aria-label", "現在地");
    crumbList.forEach(function (c, i) {
      if (i > 0) {
        var sep = doc.createElement("span");
        sep.className = "sep";
        sep.textContent = "›";
        crumbs.appendChild(sep);
      }
      // 最後の1つは「いま居る所」なので、自分自身への行き先にはしない
      if (c.path && i < crumbList.length - 1) {
        var a = doc.createElement("a");
        // pathが既に ?id=... のようなクエリを持つ呼び出し元がある
        // （どのboardから来たかを戻り先に持ち回るため）ので、? と & を使い分ける。
        var sep = c.path.indexOf("?") >= 0 ? "&" : "?";
        a.href = tokenPart ? c.path + sep + tokenPart : c.path;
        a.textContent = c.label;
        crumbs.appendChild(a);
      } else {
        var span = doc.createElement("span");
        if (i === crumbList.length - 1) span.className = "here";
        span.textContent = c.label;
        crumbs.appendChild(span);
      }
    });
    bar.appendChild(crumbs);

    var actions = doc.createElement("div");
    actions.className = "kd-subactions";
    (opt.actions || []).forEach(function (a) {
      var b = doc.createElement("button");
      b.type = "button";
      b.textContent = a.label;
      if (a.id) b.id = a.id;
      if (a.title) b.title = a.title;
      if (a.primary) b.className = "primary";
      b.addEventListener("click", a.onClick);
      actions.appendChild(b);
    });
    bar.appendChild(actions);

    var toastEl = doc.createElement("p");
    toastEl.className = "kd-toast";
    toastEl.hidden = true;
    toastEl.setAttribute("role", "status");
    bar.appendChild(toastEl);

    host.appendChild(bar);

    var timer = null;
    function hideNow() {
      if (timer) { global.clearTimeout(timer); timer = null; }
      toastEl.classList.remove("show");
      toastEl.hidden = true;
    }
    /// 出して、既定では2秒で自分から消す。出すたびに数え直すので、
    /// 続けて知らせても前の残り時間で消えることはない。
    function toast(text, ms) {
      if (!text) return;
      if (timer) { global.clearTimeout(timer); timer = null; }
      toastEl.textContent = text;
      toastEl.hidden = false;
      // hidden を外した直後に class を足しても遷移しないので、1フレーム待つ
      global.requestAnimationFrame(function () { toastEl.classList.add("show"); });
      timer = global.setTimeout(function () {
        toastEl.classList.remove("show");
        timer = global.setTimeout(function () {
          toastEl.hidden = true; timer = null;
        }, 200);
      }, ms || 2000);
    }
    return { bar: bar, toast: toast, hideToast: hideNow };
  }

  function renderNav(host, currentPath, token) {
    injectStyles(host.ownerDocument);
    var doc = host.ownerDocument;

    var wrap = doc.createElement("div");
    wrap.className = "kd-nav";

    var btn = doc.createElement("button");
    btn.type = "button";
    btn.className = "kd-nav-btn";
    btn.setAttribute("aria-haspopup", "true");
    btn.setAttribute("aria-expanded", "false");
    btn.appendChild(doc.createTextNode("KeyDeck"));
    var caret = doc.createElement("span");
    caret.className = "kd-nav-caret";
    caret.textContent = "\u25bc";
    btn.appendChild(caret);

    var menu = doc.createElement("div");
    menu.className = "kd-nav-menu";
    menu.hidden = true;

    NAV_TREE.forEach(function (dept) {
      var label = doc.createElement("div");
      label.className = "kd-nav-group";
      label.textContent = dept.dept;
      menu.appendChild(label);

      if (dept.items.length === 0) {
        var none = doc.createElement("button");
        none.type = "button";
        none.className = "kd-nav-item";
        none.disabled = true;
        none.textContent = "（未実装）";
        menu.appendChild(none);
      }
      dept.items.forEach(function (item) {
        var row = doc.createElement("button");
        row.type = "button";
        row.className = "kd-nav-item" + (item.path === currentPath ? " current" : "");
        row.textContent = (item.path === currentPath ? "● " : "　") + item.label;
        row.addEventListener("click", function () {
          var sep = item.path.indexOf("?") >= 0 ? "&" : "?";
          global.location.href = item.path + sep + "token=" + encodeURIComponent(token || "");
        });
        menu.appendChild(row);
      });
    });

    var closeTimer = null;
    function openMenu() {
      if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
      menu.hidden = false;
      btn.setAttribute("aria-expanded", "true");
      // hidden を外した直後に class を足しても動きは出ない（同じ描画で確定してしまう）。
      // 1コマ待ってから「開いた」状態にする。
      global.requestAnimationFrame(function () { menu.classList.add("open"); });
    }
    function closeMenu() {
      if (menu.hidden) return;
      menu.classList.remove("open");
      btn.setAttribute("aria-expanded", "false");
      // しぼむ動きが終わってから消す。すぐ消すと動きが見えない
      closeTimer = global.setTimeout(function () { menu.hidden = true; closeTimer = null; }, 160);
    }

    btn.addEventListener("click", function (event) {
      event.stopPropagation();
      if (menu.hidden) openMenu(); else closeMenu();
    });
    // 外を触ったら閉じる。メニューの中は閉じない
    doc.addEventListener("pointerdown", function (event) {
      if (!wrap.contains(event.target)) closeMenu();
    });
    doc.addEventListener("keydown", function (event) {
      if (event.key === "Escape") closeMenu();
    });

    wrap.appendChild(btn);
    wrap.appendChild(menu);
    host.appendChild(wrap);
  }

  function showTokenNotice(doc) {
    var d = doc || document;
    if (d.getElementById("kd-token-notice")) return;
    var box = d.createElement("div");
    box.id = "kd-token-notice";
    box.setAttribute("style", [
      "position:fixed", "inset:0", "z-index:9999",
      "background:#101526", "color:#f4f7ff",
      "font-family:system-ui,-apple-system,'Segoe UI',sans-serif",
      "display:flex", "align-items:center", "justify-content:center",
      "padding:24px", "text-align:center",
    ].join(";"));
    var card = d.createElement("div");
    card.setAttribute("style", "max-width:520px;line-height:1.9");

    var h = d.createElement("div");
    h.setAttribute("style", "font-size:17px;font-weight:700;margin-bottom:14px");
    h.textContent = "接続用のトークンがありません";
    card.appendChild(h);

    var p1 = d.createElement("p");
    p1.setAttribute("style", "font-size:13px;color:#b7bfd6;margin:0 0 14px");
    p1.textContent = "トークンはHubを起動するたびに新しく作られます。"
      + "そのため、URLだけを覚えて開くことはできません。";
    card.appendChild(p1);

    var p2 = d.createElement("p");
    p2.setAttribute("style", "font-size:13px;color:#b7bfd6;margin:0 0 6px");
    p2.textContent = "Hubの黒い画面（コンソール）に出ているURLを開いてください。";
    card.appendChild(p2);

    var p3 = d.createElement("p");
    p3.setAttribute("style", "font-size:12px;color:#6b7590;margin:0");
    p3.textContent = "端末から使うときは、PCで「QRギャラリー」を開いてQRを読み取ります。";
    card.appendChild(p3);

    box.appendChild(card);
    d.body.appendChild(box);
  }

  // ================================================================
  // 中身の見本（部品カタログ）。**押した瞬間にHubへ送るものではなく、
  // キーやDeckのタイルに入れる中身の見本**。実行できるかどうかは、保存後に
  // Hubが全JSONから作り直す許可リストが決める。
  //
  // ここに置いてあるのは、キー編集（/keys）とDeck編集（/deckedit）が
  // **同じ一覧を出す**ため。片方にだけ部品が増える状態を作らない。
  // ================================================================
  var LETTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".split("");
  var DIGITS = "0123456789".split("");
  /// vk名と表示名。**vk名は proto-keymap の VK_DICTIONARY に在るものだけ**。
  /// 辞書に無い名前をここに置くと、選んで保存した瞬間に LOAD_VK_UNKNOWN で弾かれる
  /// （2026-09-23 まで HOME/END/PGUP/PGDN/DELETE がそうなっていた）。
  var NAMED = [
    ["ENTER", "Enter"], ["SPACE", "Space"], ["BKSP", "Bksp"], ["TAB", "Tab"],
    ["ESC", "Esc"], ["SHIFT", "Shift"], ["CTRL", "Ctrl"], ["ALT", "Alt"],
    ["WIN", "Win"], ["UP", "↑"], ["DOWN", "↓"], ["LEFT", "←"], ["RIGHT", "→"],
    ["HOME", "Home"], ["END", "End"], ["PGUP", "PgUp"], ["PGDN", "PgDn"],
    ["DEL", "Del"], ["F1", "F1"], ["F2", "F2"], ["F3", "F3"], ["F4", "F4"],
    ["F5", "F5"], ["F6", "F6"], ["F7", "F7"], ["F8", "F8"], ["F9", "F9"],
    ["F10", "F10"], ["F11", "F11"], ["F12", "F12"],
  ];
  var SYMBOLS = "、。・ー！？＃＠／＊＋－＝（）「」『』【】％＆＄＾～｜＜＞：；".split("");
  var COMBOS = [
    ["コピー", ["CTRL", "C"]],
    ["貼り付け", ["CTRL", "V"]],
    ["切り取り", ["CTRL", "X"]],
    ["元に戻す", ["CTRL", "Z"]],
    ["やり直し", ["CTRL", "Y"]],
    ["全選択", ["CTRL", "A"]],
    ["保存", ["CTRL", "S"]],
    ["検索", ["CTRL", "F"]],
    ["閉じる", ["CTRL", "W"]],
    ["新規", ["CTRL", "N"]],
    ["窓の切替", ["ALT", "TAB"]],
    ["戻る", ["ALT", "LEFT"]],
    ["進む", ["ALT", "RIGHT"]],
    ["デスクトップ", ["WIN", "D"]],
    ["エクスプローラ", ["WIN", "E"]],
    ["切り取り&スケッチ", ["WIN", "SHIFT", "S"]],
  ];

  var ACTION_TABS = [
    { id: "recent", label: "最近" },
    { id: "basic", label: "文字" },
    { id: "special", label: "特殊キー" },
    { id: "shortcut", label: "ショートカット" },
    { id: "symbols", label: "記号" },
    { id: "layers", label: "レイヤー切替" },
    { id: "macros", label: "マクロ" },
  ];

  /// レイヤーの表示名。説明の1行目から「Vol1.3 」などの接頭辞を落として短くする。
  function layerName(layer) {
    var first = (layer.description || "").split("\n")[0].trim();
    if (!first) return "レイヤー" + layer.id;
    var m = first.match(/^\S+\s+(.+?)[。.]/);
    return (m ? m[1] : first).slice(0, 12);
  }

  /// 部品1つ = { label, action, note? }。タブごとに分けて返す。
  /// keymapId を渡すと、その盤のレイヤー切替キーも作る（Deck編集では省略する）。
  function buildActionParts(config, keymapId) {
    var basic = [];
    LETTERS.forEach(function (c) { basic.push({ label: c, action: { t: "key", vk: c } }); });
    DIGITS.forEach(function (d) { basic.push({ label: d, action: { t: "key", vk: d } }); });

    // 文字と混ぜると2行に収まらず、下端のものが見切れて掴めなくなるので別のタブにする
    var special = NAMED.map(function (pair) {
      return { label: pair[1], action: { t: "key", vk: pair[0] } };
    });

    var symbols = SYMBOLS.map(function (c) {
      return { label: c, action: { t: "text", string: c } };
    });

    // マクロ = すでにDeckに登録されている操作。許可リストに入っているので確実に動く
    var macros = [];
    var seen = {};
    var decks = (config && config.decks) || {};
    Object.keys(decks).forEach(function (deckId) {
      (decks[deckId].pages || []).forEach(function (page) {
        (page.slots || []).forEach(function (slot) {
          if (!slot.action || slot.action.t === "none") return;
          var key = JSON.stringify(slot.action);
          if (seen[key]) return;
          seen[key] = true;
          macros.push({ label: slot.label || "?", action: slot.action, note: decks[deckId].deckId });
        });
      });
    });

    // レイヤー切替キー。いま読み込まれているレイヤーの数だけ自動で並ぶ
    var layers = [];
    var keymap = keymapId && config && config.keymaps ? config.keymaps[keymapId] : null;
    ((keymap && keymap.layers) || []).forEach(function (layer) {
      if (layer.id === 0) return;   // 0は基盤なので切替先にしない
      var name = layerName(layer);
      layers.push({ label: name + "\n押す間", action: { t: "mo", layer: layer.id }, note: "mo" });
      layers.push({ label: name + "\n固定", action: { t: "tg", layer: layer.id }, note: "tg" });
    });

    // Win+数字は**タスクバーの左から何番目を開くか**。並び順を変えると行き先も変わる
    var shortcut = [];
    for (var n = 1; n <= 9; n += 1) {
      shortcut.push({
        label: "Win+" + n + "\nタスクバー",
        action: { t: "chord", keys: ["WIN", String(n)] },
      });
    }
    COMBOS.forEach(function (pair) {
      shortcut.push({ label: pair[0] + "\n" + pair[1].join("+"), action: { t: "chord", keys: pair[1] } });
    });

    return { basic: basic, special: special, shortcut: shortcut, symbols: symbols, macros: macros, layers: layers };
  }

  global.KDComponents = {
    applyTheme: applyTheme,
    markDefaultOption: markDefaultOption,
    syncTheme: syncTheme,
    renderJog: renderJog,
    renderNav: renderNav,
    renderSubHeader: renderSubHeader,
    showTokenNotice: showTokenNotice,
    injectStyles: injectStyles,
    resolveDisplay: resolveDisplay,
    renderLabel: renderLabel,
    renderKeyboard: renderKeyboard,
    renderDeck: renderDeck,
    fitDeck: fitDeck,
    // 中身の見本。キー編集とDeck編集が同じ一覧を出すための唯一の実装
    buildActionParts: buildActionParts,
    actionTabs: ACTION_TABS,
    layerName: layerName,
  };
})(window);
