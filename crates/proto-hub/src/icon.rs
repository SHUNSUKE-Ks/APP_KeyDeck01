//! Deckタイルのアイコン画像を `static/icons/` へ保存する（2026-09-25・ユーザー裁定。
//! 不変条件6の `static/icons/` 追加ぶん）。
//!
//! 用途: 外のアプリ（kanban-note01 など）が書き出したキャラクターの四角アイコンを、
//! Deck編集画面の「取り込み」から受け取る。
//!
//! ■ 守っていること（1つも緩めない）
//! - 書き先は `static/icons/<name>.<ext>` だけ。**ファイル名はクライアントから受け取らない**。
//!   受け取るのは `name` だけで、`[a-z0-9_]{1,64}` を通ったものしか使わない
//! - 拡張子はクライアントが決めない。**中身の先頭バイトから決める**（PNG / JPEG / WebP のみ）
//! - **SVG は受け付けない。** SVG はスクリプトを埋め込める文書で、`/icons/` は Hub と同じ
//!   オリジンで配られる。置けるようにすると、token を持つ画面を乗っ取る足場になる
//! - 1枚 1MiB まで。アイコンにそれ以上は要らない
//! - 既存があれば `.bak` へ退避してから書き、書いた後に読み直して一致を確かめる。
//!   合わなければ巻き戻す

use crate::error::{ICON_SAVE_FAILED, ICON_SAVE_REJECTED};
use std::path::{Path, PathBuf};

pub const ICONS_DIR: &str = "static/icons";
pub const MAX_ICON_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Png,
    Jpeg,
    Webp,
}

impl ImageKind {
    pub fn ext(self) -> &'static str {
        match self {
            ImageKind::Png => "png",
            ImageKind::Jpeg => "jpg",
            ImageKind::Webp => "webp",
        }
    }
}

/// 先頭バイトで画像の種類を判定する。ここに無い形式はすべて拒否する。
pub fn sniff_image(bytes: &[u8]) -> Option<ImageKind> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(ImageKind::Png);
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageKind::Jpeg);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(ImageKind::Webp);
    }
    None
}

/// ファイル名に使ってよい `name` か。**パスを組み立てる前の唯一の関門。**
pub fn icon_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconError {
    pub code: &'static str,
    pub cause: String,
}

impl IconError {
    fn rejected(cause: impl Into<String>) -> Self {
        Self { code: ICON_SAVE_REJECTED, cause: cause.into() }
    }
    fn failed(cause: impl Into<String>) -> Self {
        Self { code: ICON_SAVE_FAILED, cause: cause.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedIcon {
    pub file_name: String,
    /// deck の `slot.icon` にそのまま書ける参照先（`/icons/<file_name>`）
    pub url: String,
    pub backup: Option<PathBuf>,
}

pub fn save_icon(dir: &Path, name: &str, bytes: &[u8]) -> Result<SavedIcon, IconError> {
    if !icon_name_is_safe(name) {
        return Err(IconError::rejected(format!(
            "icon name '{name}' must be 1-64 chars of [a-z0-9_] (it becomes a file name)"
        )));
    }
    if bytes.is_empty() {
        return Err(IconError::rejected("icon body is empty"));
    }
    if bytes.len() > MAX_ICON_BYTES {
        return Err(IconError::rejected(format!(
            "icon is {} bytes; the limit is {MAX_ICON_BYTES}",
            bytes.len()
        )));
    }
    let kind = sniff_image(bytes).ok_or_else(|| {
        IconError::rejected("only PNG / JPEG / WebP are accepted (SVG is refused: it can carry script)")
    })?;

    std::fs::create_dir_all(dir)
        .map_err(|e| IconError::failed(format!("failed to create {}: {e}", dir.display())))?;

    let file_name = format!("{name}.{}", kind.ext());
    let path = dir.join(&file_name);
    let backup = dir.join(format!("{file_name}.bak"));

    let had_previous = path.exists();
    if had_previous {
        std::fs::copy(&path, &backup)
            .map_err(|e| IconError::failed(format!("failed to back up {}: {e}", path.display())))?;
    }

    let restore = || {
        if had_previous {
            let _ = std::fs::copy(&backup, &path);
        } else {
            let _ = std::fs::remove_file(&path);
        }
    };

    if let Err(e) = std::fs::write(&path, bytes) {
        restore();
        return Err(IconError::failed(format!("failed to write {}: {e}", path.display())));
    }

    // 書いた後に読み直す。途中で切れた・別物が残った、はここでしか分からない
    match std::fs::read(&path) {
        Ok(back) if back == bytes && sniff_image(&back) == Some(kind) => {}
        _ => {
            restore();
            return Err(IconError::failed(format!(
                "{} did not read back as written; rolled back",
                path.display()
            )));
        }
    }

    Ok(SavedIcon {
        url: format!("/icons/{file_name}"),
        file_name,
        backup: had_previous.then_some(backup),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13];
    const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 16];
    const WEBP: &[u8] = b"RIFF\x10\x00\x00\x00WEBPVP8 ";

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kd_icon_test_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn sniff_accepts_png_jpeg_webp_only() {
        assert_eq!(sniff_image(PNG), Some(ImageKind::Png));
        assert_eq!(sniff_image(JPEG), Some(ImageKind::Jpeg));
        assert_eq!(sniff_image(WEBP), Some(ImageKind::Webp));
        assert_eq!(sniff_image(b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script/></svg>"), None);
        assert_eq!(sniff_image(b"GIF89a"), None);
        assert_eq!(sniff_image(b""), None);
    }

    #[test]
    fn name_gate_refuses_paths() {
        assert!(icon_name_is_safe("chara_mia_01"));
        for bad in ["", "../x", "a/b", "a\\b", "a.png", "Mia", "a b", &"x".repeat(65)] {
            assert!(!icon_name_is_safe(bad), "{bad:?} must be refused");
        }
    }

    #[test]
    fn saves_with_extension_from_content_and_returns_url() {
        let dir = temp_dir("save");
        let saved = save_icon(&dir, "chara_mia", PNG).expect("png must save");
        assert_eq!(saved.file_name, "chara_mia.png");
        assert_eq!(saved.url, "/icons/chara_mia.png");
        assert_eq!(saved.backup, None);
        assert_eq!(std::fs::read(dir.join("chara_mia.png")).unwrap(), PNG);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn overwrite_keeps_a_backup_of_the_previous_file() {
        let dir = temp_dir("overwrite");
        save_icon(&dir, "hero", PNG).unwrap();
        let mut second = PNG.to_vec();
        second.push(7);
        let saved = save_icon(&dir, "hero", &second).unwrap();
        assert_eq!(saved.backup, Some(dir.join("hero.png.bak")));
        assert_eq!(std::fs::read(dir.join("hero.png.bak")).unwrap(), PNG);
        assert_eq!(std::fs::read(dir.join("hero.png")).unwrap(), second);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refuses_svg_bad_names_empty_and_oversize_without_touching_disk() {
        let dir = temp_dir("refuse");
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
        let err = save_icon(&dir, "evil", svg).unwrap_err();
        assert_eq!(err.code, ICON_SAVE_REJECTED);
        assert_eq!(save_icon(&dir, "../evil", PNG).unwrap_err().code, ICON_SAVE_REJECTED);
        assert_eq!(save_icon(&dir, "empty", b"").unwrap_err().code, ICON_SAVE_REJECTED);
        let mut big = PNG.to_vec();
        big.resize(MAX_ICON_BYTES + 1, 0);
        assert_eq!(save_icon(&dir, "big", &big).unwrap_err().code, ICON_SAVE_REJECTED);
        assert!(!dir.exists(), "a refused request must not even create the directory");
    }
}
