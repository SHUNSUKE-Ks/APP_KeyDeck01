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
    ".kd-surface .slot .lb { font-size: 10px; line-height: 1.15; color: var(--kd-ink-muted); overflow: hidden; }",
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

  function renderLabel(btn, label) {
    btn.textContent = "";
    var lines = String(label == null ? "" : label).split("\n");
    if (lines.length >= 2) {
      var small = btn.ownerDocument.createElement("small");
      small.textContent = lines[0];
      btn.appendChild(small);
      btn.appendChild(btn.ownerDocument.createTextNode(lines.slice(1).join("\n")));
      // 上下の主従を入れ替えるのは「小さい行が日本語側の代替表記」のときだけ。
      // vol1.3のQ〜Pは小さい行が数字ヒントなので、入れ替えると数字が主になって読めない。
      if (!/^[0-9]+$/.test(lines[0])) btn.classList.add("dual");
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
  function renderKeyboard(host, keymap, state, options) {
    var opt = options || {};
    injectStyles(host.ownerDocument);
    host.classList.add("kd-surface");
    if (!opt.interactive && !opt.editable) host.classList.add("kd-preview");

    if (!keymap || !keymap.board) {
      oops(host, "キーボードが見つかりません");
      return null;
    }

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

        var ico = doc.createElement("span");
        ico.className = "ico";
        if (slot.icon) {
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

  global.KDComponents = {
    showTokenNotice: showTokenNotice,
    injectStyles: injectStyles,
    resolveDisplay: resolveDisplay,
    renderLabel: renderLabel,
    renderKeyboard: renderKeyboard,
    renderDeck: renderDeck,
    fitDeck: fitDeck,
  };
})(window);
