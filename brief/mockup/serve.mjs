// モック配信用の小さな静的サーバー。
//
// ■ なぜ python -m http.server をやめたか
//   素のhttp.serverは Last-Modified だけを返す。ブラウザはこれを根拠に
//   ヒューリスティックなキャッシュを効かせるため、**モックを直してもリロードで
//   古いページが出続ける**。実際に、直した直後の版を見ているつもりで
//   修正前の壊れた画面を評価してしまう事故が起きた。
//   Hub本体は同じ理由で T29 に Cache-Control: no-store を入れている。ここだけ
//   抜けていた。
//
// ■ やること
//   このフォルダのファイルを返す。すべての応答に no-store を付ける。それだけ。
//   モックは開発中の使い捨てなので、キャッシュして得することが何も無い。

import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join, normalize, sep } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL(".", import.meta.url));
const PORT = Number(process.argv[2] ?? 5212);

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
};

function send(res, status, body, type) {
  res.writeHead(status, {
    "content-type": type ?? "text/plain; charset=utf-8",
    // ここが本題
    "cache-control": "no-store",
  });
  res.end(body);
}

const server = createServer(async (req, res) => {
  // クエリ（?cb=… 等）を落とす。パスだけを見る
  const rawPath = decodeURIComponent(new URL(req.url, "http://localhost").pathname);
  // ルートは一覧ではなくモック本体へ寄せる（開くたびにファイル名を打たなくてよい）
  const target = rawPath === "/" ? "/mock_p005_stage_d_layout_editor.html" : rawPath;

  // ROOTの外へ出さない。`..` を含むパスは正規化してから前方一致で弾く
  const full = normalize(join(ROOT, target));
  if (!full.startsWith(ROOT.endsWith(sep) ? ROOT : ROOT + sep)) {
    send(res, 403, "forbidden");
    return;
  }

  try {
    const info = await stat(full);
    if (!info.isFile()) {
      send(res, 404, "not found");
      return;
    }
    const body = await readFile(full);
    send(res, 200, body, TYPES[extname(full).toLowerCase()]);
  } catch (error) {
    send(res, 404, "not found: " + target);
  }
});

server.listen(PORT, () => {
  console.log(`mockup server: http://localhost:${PORT}/  (Cache-Control: no-store)`);
  console.log(`  root: ${ROOT}`);
});
