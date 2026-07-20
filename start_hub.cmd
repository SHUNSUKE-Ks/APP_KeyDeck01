@echo off
REM start_hub.cmd — D26: サーバー自体の起動はブラウザからは物理的に不可能なため、
REM ダブルクリックでHub(proto-hub)を起動できるスクリプトをリポジトリ直下に同梱する。
REM cdをこのスクリプト自身の場所（=workspace root）にしてから起動する。
cd /d "%~dp0"
echo KeyDeck Hub (proto-hub) を起動します...
echo 作業フォルダ: %cd%
cargo run -p proto-hub
pause
