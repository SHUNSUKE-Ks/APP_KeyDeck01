//! アプリ起動の許可リスト（`apps/apps.json`）。
//!
//! ## なぜこの形なのか（2026-09-12・ユーザー裁定）
//!
//! CLAUDE.md 不変条件6は「任意コマンド実行APIは絶対に作らない」と定めている。
//! ここで作るのは**任意ではない**。起動できるのは `apps/apps.json` に**PC側で**
//! 書いたものだけで、クライアントが送れるのは登録済みidだけ（不変条件1と同じ形）。
//!
//! 守っていること:
//!
//! - **実行ファイルのパスも引数も、クライアントから受け取らない。** JSONだけが決める
//! - **シェルを経由しない。** `cmd /C` を使わず `Command::new(exe)` へ直接渡すので、
//!   パスや引数に `&` `|` `>` が入っていても別のコマンドにはならない
//! - 起動できるのは `.exe` だけ。`.bat`/`.cmd`/`.ps1` は**スクリプト解釈系**であり、
//!   1行足すだけで任意コマンド実行に化けるので拒否する
//! - `exe` は絶対パスのみ。相対パスを許すと、Hubの作業ディレクトリ次第で
//!   どれが起動するかが変わる（同名の実行ファイルを掴まされる余地ができる）
//!
//! 実在確認は**起動時と発火時の両方**で行う。起動時だけでは、その後アプリを
//! 消した／動かしたときに嘘のボタンが残る。

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

pub const LOAD_APP_SCHEMA_INVALID: &str = "LOAD_APP_SCHEMA_INVALID";
pub const LOAD_APP_EXE_INVALID: &str = "LOAD_APP_EXE_INVALID";
pub const APP_LAUNCH_UNKNOWN: &str = "APP_LAUNCH_UNKNOWN";
pub const APP_LAUNCH_FAILED: &str = "APP_LAUNCH_FAILED";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    pub code: &'static str,
    pub cause: String,
}

impl AppError {
    pub fn new(code: &'static str, cause: impl Into<String>) -> Self {
        Self {
            code,
            cause: cause.into(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.cause)
    }
}

/// 起動してよいアプリ1件。`exe`と`args`は検証済み。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDef {
    /// 画面に出す名前。人が読む用で、起動には使わない。
    pub label: String,
    pub exe: String,
    pub args: Vec<String>,
}

/// `appId -> AppDef`。クライアントはここに載っているidしか名乗れない。
#[derive(Debug, Clone, Default)]
pub struct AppRegistry {
    apps: BTreeMap<String, AppDef>,
}

impl AppRegistry {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get(&self, id: &str) -> Option<&AppDef> {
        self.apps.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.apps.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.apps.len()
    }

    /// 画面へ配る一覧（id と label だけ）。**exeとargsは絶対に外へ出さない。**
    /// 端末にPCの中の配置を教える必要はなく、教えれば攻撃者への地図になる。
    pub fn manifest(&self) -> Vec<(&str, &str)> {
        self.apps
            .iter()
            .map(|(id, def)| (id.as_str(), def.label.as_str()))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppFileRoot {
    /// 人向けの覚書。中身は使わない。
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
    apps: Vec<AppEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppEntry {
    id: String,
    label: String,
    exe: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
}

/// `[a-z0-9_]{1,64}`。layoutIdと同じ規則で揃える。
/// idはパスの組み立てには使わないが、ログと画面に出るので同じ厳しさにしておく。
fn app_id_is_safe(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Windowsの絶対パスか（`C:\...` か UNC `\\server\share`）。
fn is_absolute_windows_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    if path.starts_with("\\\\") {
        return true;
    }
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

pub fn load_app_registry(apps_dir: &Path) -> Result<AppRegistry, AppError> {
    let path = apps_dir.join("apps.json");
    // 置いていなければ「起動できるアプリは無い」。機能を使わない人の起動を止めない。
    if !path.is_file() {
        return Ok(AppRegistry::empty());
    }
    let text = std::fs::read_to_string(&path).map_err(|error| {
        AppError::new(
            LOAD_APP_SCHEMA_INVALID,
            format!("{}: failed to read file: {error}", path.display()),
        )
    })?;
    load_app_registry_str(&path.display().to_string(), &text)
}

/// 文字列からレジストリを構築する（単体テスト用。ディスクI/O無し）。
///
/// **`exe`の実在は見ない。** 実在はディスクの話で、形の検証とは別。
/// 実在しないものは起動時に警告し、押されたときに code+cause で断る
/// （消したアプリ1つでHubが起動しなくなるのは、直し方が分からない止まり方になる）。
pub fn load_app_registry_str(source: &str, text: &str) -> Result<AppRegistry, AppError> {
    let root: AppFileRoot = serde_json::from_str(text)
        .map_err(|error| AppError::new(LOAD_APP_SCHEMA_INVALID, format!("{source}: {error}")))?;

    let mut apps: BTreeMap<String, AppDef> = BTreeMap::new();
    for entry in root.apps {
        if !app_id_is_safe(&entry.id) {
            return Err(AppError::new(
                LOAD_APP_SCHEMA_INVALID,
                format!(
                    "{source}: app id '{}' must match [a-z0-9_] and be 1..=64 chars",
                    entry.id
                ),
            ));
        }
        if apps.contains_key(&entry.id) {
            return Err(AppError::new(
                LOAD_APP_SCHEMA_INVALID,
                format!("{source}: duplicate app id '{}'", entry.id),
            ));
        }
        if entry.label.trim().is_empty() {
            return Err(AppError::new(
                LOAD_APP_SCHEMA_INVALID,
                format!("{source}: app '{}': label must not be empty", entry.id),
            ));
        }

        // **パスを受け入れる前の関門。** ここを緩めると任意コマンド実行になる。
        if !is_absolute_windows_path(&entry.exe) {
            return Err(AppError::new(
                LOAD_APP_EXE_INVALID,
                format!(
                    "{source}: app '{}': exe must be an absolute path (got '{}')",
                    entry.id, entry.exe
                ),
            ));
        }
        if !entry.exe.to_ascii_lowercase().ends_with(".exe") {
            return Err(AppError::new(
                LOAD_APP_EXE_INVALID,
                format!(
                    "{source}: app '{}': exe must end with .exe (got '{}'); \
                     .bat/.cmd/.ps1 are script interpreters and are refused",
                    entry.id, entry.exe
                ),
            ));
        }

        apps.insert(
            entry.id,
            AppDef {
                label: entry.label,
                exe: entry.exe,
                args: entry.args,
            },
        );
    }
    Ok(AppRegistry { apps })
}

/// 登録済みのアプリを起動する。
///
/// **シェルを経由しない。** `Command::new(exe)` へ実行ファイルと引数を別々に渡すので、
/// 引数に `&` や `|` が入っていても別のコマンドにはならない。
pub fn launch(registry: &AppRegistry, id: &str) -> Result<(), AppError> {
    let Some(def) = registry.get(id) else {
        return Err(AppError::new(
            APP_LAUNCH_UNKNOWN,
            format!("app '{id}' is not in apps/apps.json"),
        ));
    };
    // 起動時に在っても、その後に消された／動かされた可能性がある。押された時に見る。
    if !Path::new(&def.exe).is_file() {
        return Err(AppError::new(
            APP_LAUNCH_FAILED,
            format!("app '{id}': {} does not exist", def.exe),
        ));
    }
    std::process::Command::new(&def.exe)
        .args(&def.args)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            AppError::new(
                APP_LAUNCH_FAILED,
                format!("app '{id}': failed to start {}: {error}", def.exe),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(text: &str) -> Result<AppRegistry, AppError> {
        load_app_registry_str("test", text)
    }

    #[test]
    fn loads_a_valid_entry() {
        let reg = load(
            r#"{ "apps": [
                { "id": "unity", "label": "Unity",
                  "exe": "C:\\Program Files\\Unity\\Editor\\Unity.exe",
                  "args": ["-projectPath", "C:\\work\\game"] }
            ] }"#,
        )
        .expect("valid file must load");
        assert_eq!(reg.len(), 1);
        let def = reg.get("unity").expect("unity must be registered");
        assert_eq!(def.args, vec!["-projectPath", "C:\\work\\game"]);
        // 画面へ出るのは id と label だけ
        assert_eq!(reg.manifest(), vec![("unity", "Unity")]);
    }

    #[test]
    fn missing_file_is_not_an_error() {
        let reg = load_app_registry(Path::new("no_such_dir_for_apps"))
            .expect("missing apps.json must not block startup");
        assert_eq!(reg.len(), 0);
    }

    /// **これが一番大事なテスト。** .bat を許すと、そのファイルに1行足すだけで
    /// 任意のコマンドが動くようになり、許可リストの意味が消える。
    #[test]
    fn refuses_script_interpreters() {
        for exe in [
            "C:\\tools\\run.bat",
            "C:\\tools\\run.cmd",
            "C:\\tools\\run.ps1",
            "C:\\tools\\RUN.BAT",
        ] {
            let text = format!(
                r#"{{ "apps": [ {{ "id": "x", "label": "x", "exe": "{}" }} ] }}"#,
                exe.replace('\\', "\\\\")
            );
            let err = load(&text).expect_err("script files must be refused");
            assert_eq!(err.code, LOAD_APP_EXE_INVALID, "exe={exe}");
        }
    }

    #[test]
    fn refuses_relative_exe() {
        let err = load(r#"{ "apps": [ { "id": "x", "label": "x", "exe": "notepad.exe" } ] }"#)
            .expect_err("relative path must be refused");
        assert_eq!(err.code, LOAD_APP_EXE_INVALID);
    }

    #[test]
    fn refuses_bad_id() {
        let err = load(
            r#"{ "apps": [ { "id": "Unity Hub", "label": "x", "exe": "C:\\a\\b.exe" } ] }"#,
        )
        .expect_err("id with spaces/uppercase must be refused");
        assert_eq!(err.code, LOAD_APP_SCHEMA_INVALID);
    }

    #[test]
    fn refuses_duplicate_id() {
        let err = load(
            r#"{ "apps": [
                { "id": "a", "label": "1", "exe": "C:\\a\\b.exe" },
                { "id": "a", "label": "2", "exe": "C:\\a\\c.exe" }
            ] }"#,
        )
        .expect_err("duplicate id must be refused");
        assert_eq!(err.code, LOAD_APP_SCHEMA_INVALID);
    }

    #[test]
    fn refuses_unknown_fields() {
        let err = load(
            r#"{ "apps": [ { "id": "a", "label": "1", "exe": "C:\\a\\b.exe", "shell": true } ] }"#,
        )
        .expect_err("unknown fields must be refused");
        assert_eq!(err.code, LOAD_APP_SCHEMA_INVALID);
    }

    #[test]
    fn launching_an_unregistered_id_is_refused() {
        let reg = AppRegistry::empty();
        let err = launch(&reg, "unity").expect_err("unknown id must be refused");
        assert_eq!(err.code, APP_LAUNCH_UNKNOWN);
    }

    /// 登録されていても実体が無ければ起動しない（起動時から状況が変わっている場合）。
    #[test]
    fn launching_a_missing_exe_is_refused_without_spawning() {
        let reg = load(
            r#"{ "apps": [ { "id": "gone", "label": "gone", "exe": "C:\\no\\such\\app.exe" } ] }"#,
        )
        .expect("shape is valid");
        let err = launch(&reg, "gone").expect_err("missing exe must be refused");
        assert_eq!(err.code, APP_LAUNCH_FAILED);
    }
}
