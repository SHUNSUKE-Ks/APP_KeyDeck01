// 読本屋への納品データを作る: 本文を1ファイルに結合し、目録 JSON を書き、画像を複製する。
// 使い方: node build_delivery.mjs <書籍フォルダ>
import { readFileSync, writeFileSync, mkdirSync, copyFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const root = process.argv[2];
const out = join(root, "納品_読本屋");
mkdirSync(join(out, "images"), { recursive: true });

const order = [
  "00_この本について.md",
  ...Array.from({ length: 14 }, (_, i) => `chapter-${String(i + 1).padStart(2, "0")}.md`),
  "appendix-a_用語集.md",
  "appendix-b_逆引き索引.md",
  "appendix-c_コード索引.md",
  "appendix-d_未確認事項.md",
  "appendix-e_確認問題の解答.md",
];

const parts = order.map((f) => readFileSync(join(root, f), "utf8").trimEnd());
const body = parts.join("\n\n") + "\n";
writeFileSync(join(out, "book.md"), body, "utf8");

for (const f of readdirSync(join(root, "images"))) copyFileSync(join(root, "images", f), join(out, "images", f));

// 目次は見出しから起こす（手で書くとずれるため）
const toc = [];
for (const line of body.split("\n")) {
  const m = line.match(/^# (第[IV]+部　.+|第\d+章　.+|付録 [A-E]　.+)$/);
  if (m) toc.push(m[1]);
}

const book = {
  workId: "keydeck-rust-websocket-input-device",
  revision: 1,
  siteId: "yomihonya",
  kind: "book",
  synthetic: false,
  status: "pending",
  title: "Rust＋WebSocketで作る自作キーボード／Stream Deck",
  subtitle: "ブラウザを入力デバイスに変える",
  authorPolicy: "筆名で掲載する方針（2026-09-17 著者本人が選択）。**筆名そのものが未提示のため、ここは未確定**。本文は AI（Claude Code / Opus 5）が執筆したことを奥付等で明記する方針を提案する",
  format: "markdown",
  bodyFile: "book.md",
  assets: readdirSync(join(root, "images")).map((f) => `images/${f}`),
  manuscriptState: "complete",
  manuscriptStateNote: "本文・付録は完成候補。独立した監修は未実施（未確認事項を参照）",
  toc,
  rights: {
    canAuthorPermitPublication: "確認済み。2026-09-17、著者本人が『本文・コード例・図版・引用資料すべてを全文そのまま掲載可』と回答した",
    code: "引用コードは題材リポジトリ内の KeyDeck 独自部分のみ。外部から取り込んだ crates/hub-core のコードは引用していない",
    images: "題材アプリの画面を執筆時に撮影したもの（5点）。第三者の画像は含まない",
    quotations: "外部文献からの引用なし",
    localPaths: "本文に C:\\ で始まる文字列が7か所あるが、すべて伏字（<user>・<アプリ>・<Windows のフォルダ>）か、題材のテストコードに書かれた架空のパス（C:\\tools\\run.bat 等）。実在する個人のパスは含まない",
    unresolved: ["著者表記（筆名の文字列そのもの）"],
  },
  unverified: [
    "独立した監修が未実施（設計と執筆を同一の AI が行った）",
    "画面写真は PC 上のヘッドレス Chrome で撮影。iPad 実機での表示は未確認",
    "iPad・Android 実機での操作感、トラックボールのジェスチャー閾値",
    "text 操作の文字が意図した入力欄に入ること",
    "英数・日本語切替の他 IME での動作。表示は送信回数からの推定",
    "トラックボールの送信頻度の上限を Hub 側でも行っているか",
    "第13章の2番・4番は開発記録からの引用で、執筆時に再現していない",
    "技術情報は 2026-09-15 時点。コードの変更で行番号・件数は変わる",
  ],
  description: {
    forWhom: "自分用の入力デバイスや操作パネルを作りたい、コードを読み慣れていない人から初級の開発者",
    problem: "ネットワーク越しに PC へ入力を送る道具を、他人に悪用される穴を作らずに設計する考え方が、手順書型の記事からは学べない",
    afterReading: [
      "端末に送らせてよいものを先に決め、その境界の内側で機能を足す設計を説明できる",
      "ボタン1回の押下が、JSON・起動時検査・WebSocket・レイヤー解決・Windows 入力を通る流れを追える",
      "「状態が変わる」と「入力が出る」を区別して、状態駆動の画面を読める",
      "境界に触れる機能（アプリ起動など）を、条件とテストで固定して足す手順を真似できる",
    ],
  },
  // 読本屋の台帳（投稿規定 v0.1 の必須カラム）に載せるときの提案。
  // v0.1 の type は マンガ/ネーム/小説 の3種で、技術書が無い。**受入側で決めること。**
  ledgerProposal: {
    ID: "YH-TEC-001（採番規則は受入側で確認）",
    Title: "Rust＋WebSocketで作る自作キーボード／Stream Deck",
    Genre: "技術, 入力デバイス, Rust",
    type: "未定。投稿規定 v0.1 の enum（マンガ / ネーム / 小説）に技術書が無い",
    作成日: "2026-09-15",
    更新日: "2026-09-17",
    note: "本文は chapter 形式ではなく book.md 1ファイル。章ごとに分けて納めることも可能（20_書籍_初版 に chapter-01..14.md がある）",
  },
  verification: {
    confirmedOn: "2026-09-15",
    tests: "cargo test --workspace 139 passed / 0 failed",
    screenshots: "同じ /ipad 画面を開いたまま、別の WebSocket 接続から実際の key.press を送って撮影",
    automatedChecks: ["個人情報・絶対パス・token・IP の検査", "確認問題と解答の番号の一致（43問）", "画像の実在", "図の JSON の schema 検証"],
  },
};
writeFileSync(join(out, "book.json"), JSON.stringify(book, null, 2) + "\n", "utf8");

console.log("chapters+appendices:", order.length, "toc entries:", toc.length, "body bytes:", Buffer.byteLength(body));
console.log("function cards:", (body.match(/^### 関数カード/gm) || []).length);
console.log(toc.join("\n"));
