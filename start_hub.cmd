@echo off
chcp 65001 >nul
REM ↑ このファイルは UTF-8。コマンドプロンプトは既定で Shift_JIS として読むため、
REM   切り替えないと日本語の行が壊れて「〜は認識されていません」が出る（実際に出た）。
REM   日本語は必ずこの行より下に書くこと。
REM start_hub.cmd — D26: サーバー自体の起動はブラウザからは物理的に不可能なため、
REM ダブルクリックでHub(proto-hub)を起動できるスクリプトをリポジトリ直下に同梱する。
REM cdをこのスクリプト自身の場所（=workspace root）にしてから起動する。
REM cargo run なので、ソースを変えていれば起動の前に作り直す（exe を直接起動するとそれが効かない）。
cd /d "%~dp0"
echo KeyDeck Hub (proto-hub) を起動します...
echo 作業フォルダ: %cd%
cargo run -p proto-hub
pause
