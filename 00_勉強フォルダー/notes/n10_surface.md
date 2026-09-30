# n10 — `crates/proto-hub/src/surface.rs`（645行）

## 役割

`surfaces/*.json`（n01で見たトラックボールの定義）を読み込み・検証し、
`SurfaceRegistry { id -> SurfaceDef }` を作るモジュール。設計決定D28
（「連続値を出す面は送ってよいが、出口はHub側のJSONだけが決める」）を
実際にコードとして体現している場所。

## 前提

- `CLAUDE.md`のD28を先に読んでおくこと。「クライアントは何に繋がるかを
  指定できない」という一文が、このファイルの`ALLOWED_BINDING_TYPES`と直結する。

## 読みどころ

- 8〜9行目のコメント: `binding.t`の許可リストは**このファイル内の固定リストのみ**
  （JSON側からは拡張できない）。n01で見た`surfaces/trackball.json`の
  `"binding": {"t":"mouse.move"}`という値がここでチェックされる。
- 20〜30行目 D9エラーコード一覧（`SURFACE_UNKNOWN_ID` `SURFACE_STATE_RANGE`
  `LOAD_SURFACE_SCHEMA_INVALID` `LOAD_SURFACE_BINDING_UNKNOWN`
  `SURFACE_GESTURE_UNKNOWN_ID` `SURFACE_GESTURE_EDGE_REQUIRED`
  `LOAD_SURFACE_GESTURE_INVALID`）… surface関連のエラーはここが正。
- 35行目 `const ALLOWED_BINDING_TYPES: &[&str] = &["mouse.move", "mouse.scroll"];`
  … **これがD28の実体**。ここに無い文字列が`binding.t`に来たらロード時に拒否される
  （33〜34行目のコメントに、後からスクロール用に追加した経緯がある）。
- 37〜40行目 `CLAMP_DEFAULT` `CLAMP_MIN` `CLAMP_MAX` … クランプの許容範囲
  （1〜1000、省略時200）。n01の`surfaces/trackball.json`の`"clamp": 200`と対応。
- 70〜100行目 `ClickButton` / `GestureAction` / `SurfaceDef` … 1つのsurfaceが
  持つデータ（binding種別・clamp・gestures）。
- 101〜129行目 `SurfaceRegistry` … 複数のsurfaceを`id`で引けるマップ。
- 215〜308行目 `load_surface_registry` / `load_surface_registry_str` … ファイルを
  読んでJSONをパースし、`ALLOWED_BINDING_TYPES`との照合、clamp範囲チェック、
  gesture定義の検証までを行う。
- 309行目〜 `parse_gesture_wire` … n01で見た`"tap1"` `"dtap1"` `"tap2"` `"tap3"`
  `"hold1"`などのgestureIdごとの定義（クリック/ダブルクリック/キー送出など）を
  パースする部分。

## なぜこうなっているか

- 「binding.tの許可リストはコード内固定」という制約が、D28の
  「クライアントは何に繋がるかを指定できない」を保証する唯一の手段になっている。
  もしこれをJSON側の自由記述にしてしまうと、理論上は`surfaces/*.json`を書き換える
  だけで任意の出口を作れてしまい、不変条件1の意図が骨抜きになる。
  （＝JSONで自由に配置・組み合わせを変えられるのは良いが、「何が出口として
  存在しうるか」の集合自体はコードでロックされている、という2段構え。）
- clampの上限・下限を持たせているのは、トラックボールの物理的な動きの量を
  超えるような異常値（バグ・悪意ある入力）がそのままマウス移動量として
  Windowsへ送られるのを防ぐため。

## 理解度チェック

1. `surfaces/trackball.json`に`"binding": {"t": "mouse.click"}`という新しい値を
   書いても、なぜ動かない（起動時に拒否される）のか？
2. `clamp`が1〜1000の範囲外だとどう扱われるか？
3. D28が「一般のUSB HIDマウスと同じ形だからリスクの増分が小さい」と言っている
   理由を、`delta.dx/dy`という設計と結びつけて説明できるか？
