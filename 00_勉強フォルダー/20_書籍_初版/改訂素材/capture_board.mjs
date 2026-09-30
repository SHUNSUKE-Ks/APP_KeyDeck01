// ボードをCDPで撮る（単発の --screenshot では iframe が描かれないことがあるため）。
// 使い方: node capture_board.mjs <token> <layoutId> <出力ファイル>
import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";

const [token, layoutId, outFile] = process.argv.slice(2);
const CHROME = "C:/Program Files/Google/Chrome/Application/chrome.exe";
const PORT = 9335;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const chrome = spawn(CHROME, ["--headless=new", `--remote-debugging-port=${PORT}`, "--window-size=1180,820",
  "--user-data-dir=" + outFile + ".profile", "about:blank"], { stdio: "ignore" });
let targets;
for (let i = 0; i < 40; i++) {
  try { targets = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json(); break; } catch { await sleep(250); }
}
const page = targets.find((t) => t.type === "page");
const cdp = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r) => (cdp.onopen = r));
let seq = 0; const waiting = new Map();
cdp.onmessage = (e) => { const m = JSON.parse(e.data); if (m.id && waiting.has(m.id)) { waiting.get(m.id)(m.result); waiting.delete(m.id); } };
const call = (method, params = {}) => new Promise((r) => { const id = ++seq; waiting.set(id, r); cdp.send(JSON.stringify({ id, method, params })); });

await call("Page.enable");
await call("Page.navigate", { url: `http://127.0.0.1:8770/layout?id=${layoutId}&token=${token}` });
await sleep(6000);
const info = await call("Runtime.evaluate", {
  expression: "[...document.querySelectorAll('iframe.tbframe')].map(f=>f.src.replace(/token=[^&]*/,'token=<token>')+' / '+f.clientWidth+'x'+f.clientHeight).join(' ; ') || '(iframe なし)'",
  returnByValue: true });
console.log("iframe:", info.result.value);
const { data } = await call("Page.captureScreenshot", { format: "png" });
writeFileSync(outFile, Buffer.from(data, "base64"));
console.log("saved", outFile);
cdp.close(); chrome.kill(); process.exit(0);
