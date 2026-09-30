# n26 — `static/trackball.html`（1031行）

## 役割

トラックボール面（T13、`/trackball?token=…`）。D28（連続値を出す面）を
実際に動かしている、**Core/View/Sinkの3層構造**の見本。頭コメント18〜27行目が
このファイル最大の学びどころ:

> ① Core … 回した結果をStateとして持つ。描画もWSも知らない
> ② View … Coreを読んで描くだけ。Stateを書き換えない
> ③ Sink … Stateの出口。登録した数だけ呼ばれる。付け替えはここだけ

「マウスという語が①②に一切出てこないのが分離できている証拠」という
一文が、良い設計の見分け方そのものを教えてくれる。

## 前提

- n07（protocol.rsの`SurfaceState`/`DeltaWire`/`SpinWire`）と、n10
  （surface.rsの許可リスト）を読んでいること。頭コメント29〜34行目の
  State形（`{delta, spin, active}`）はn07のワイヤー形式とほぼ同じ形で
  対応している。

## 読みどころ

- 413〜460行目 **Core**（`const Core = (function () {...})();`）:
  `axisAngle` / `multiply`（3x3回転行列の合成） / `toQuat` / `normalize` /
  `rotateBy(dx, dy)` / `emit(delta)`。回転を「クォータニオン」で累積し、
  ドラッグ量から回転行列を作って左から掛ける、という計算（頭コメント
  36〜39行目の「3Dに見せている仕組み」に対応）。**マウスやWebSocketの語彙が
  一切出てこないことを確認しながら読む**のがこのセクションの正しい読み方。
- 521〜660行目 **View**（`const View = (function () {...})();`）:
  `resize` / `rebuildPattern`（フィボナッチ配置の点群） / `drawRing` / `draw` /
  `setRaw` / `render`。Coreの状態を読んで球面と大円を描くだけで、
  Stateを書き換える処理は無い（コメント通りの一方通行）。
- 688〜761行目 **Sink**（WS接続部分。名前としての`Sink`ブロックは無いが、
  役割としてここが③に当たる）: `setConnectionBadge` / `connect()`
  （705行目`new WebSocket`） / `scheduleReconnect` / `clamp(v)` /
  `clampScroll(v)`。**クランプの上限値はサーバ側（n10のCLAMP_MAX）とは別に
  クライアント側にも存在する**点に注意（送る前に丸めても、Hub側の
  検証は独立して行われる＝多層防御）。
- 865〜1013行目 `dist` / `beginDrag` / `endDrag` / `clearHoldTimer` /
  `fireHoldDown` / `fireHoldUp` / `completeSingleTap` /
  `finalizeSessionIfEmpty` / `onPointerDown` / `onPointerMove` / `onPointerUp` /
  `frame()` … タップ・ダブルタップ・長押しなどの**ジェスチャー判定**
  （n07の`SurfaceGesture`を送る側）。1本の指の動きから「タップか」
  「長押しか」「ドラッグか」を時間・距離で判定するステートマシン。

## なぜこうなっているか

- ①②を「何を操作しているか知らない」形にしておくと、将来③（Sink）だけを
  差し替えれば、同じ球面UIをマウス以外の用途（3D角度の入力、パッド入力等）
  にも転用できる。実際、頭コメント29〜34行目のState形は
  `delta`（マウス向け・今回使用）だけでなく`spin`（3D角度向け・将来）・
  `active`（パッド向け・将来）も持っている＝**今使わない拡張の余地を、
  型として先に用意してある**設計。
- クランプがクライアント・サーバ両方にあるのは、片方が壊れても
  （あるいは悪意ある改造クライアントが来ても）もう片方が守る、という
  多層防御の考え方。

## 理解度チェック

1. Core（①）のコードの中に「mouse」という単語が出てこないことは、
   何を保証しているか？
2. `spin`と`active`は今のバージョンで実際に使われているか？使われていない
   なら、なぜ存在するのか？
3. クランプがクライアント側とサーバ側（n10）の両方にあるのはなぜか。
   どちらか一方だけで十分ではないのか？
