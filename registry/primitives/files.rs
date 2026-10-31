use std::{collections::HashMap, sync::LazyLock};

use gpui::{App, Global};

use super::LucideIcon;

/// Which icon a file shows: an owner's rules by name and by extension, laid over Mok's map. Without one, Mok's map alone.
#[derive(Clone, Default)]
pub struct IconTheme {
    names: HashMap<String, LucideIcon>,
    extensions: HashMap<String, LucideIcon>,
}

impl Global for IconTheme {}

impl IconTheme {
    /// A file of this exact name, in any case: `Cargo.toml`, `Dockerfile`.
    pub fn name(mut self, name: &str, icon: LucideIcon) -> Self {
        self.names.insert(name.to_lowercase(), icon);
        self
    }

    /// Files ending in this extension, given without its dot.
    pub fn extension(mut self, extension: &str, icon: LucideIcon) -> Self {
        self.extensions.insert(extension.to_lowercase(), icon);
        self
    }

    /// Serves every window from now on.
    pub fn apply(self, cx: &mut App) {
        log::info!(
            "icon theme: {} names, {} extensions",
            self.names.len(),
            self.extensions.len()
        );
        cx.set_global(self);
        cx.refresh_windows();
    }

    pub fn file(&self, name: &str) -> LucideIcon {
        let name = name.to_lowercase();
        let extension = name.rsplit_once('.').map(|(_, extension)| extension);
        self.names
            .get(&name)
            .or_else(|| extension.and_then(|extension| self.extensions.get(extension)))
            .copied()
            .unwrap_or_else(|| mok_icon(extension))
    }
}

static MOK: LazyLock<IconTheme> = LazyLock::new(IconTheme::default);

/// The owner's icon theme, or Mok's map alone.
pub fn icon_theme(cx: &App) -> &IconTheme {
    cx.try_global::<IconTheme>().unwrap_or(&MOK)
}

/// The icon a file's name suggests, under the owner's icon theme.
pub fn file_icon(name: &str, cx: &App) -> LucideIcon {
    icon_theme(cx).file(name)
}

/// Mok's icon for a file by its extension, in lower case.
fn mok_icon(extension: Option<&str>) -> LucideIcon {
    match extension {
        Some(
            "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "swift" | "c" | "h" | "cpp" | "java"
            | "kt" | "rb" | "php" | "css" | "html",
        ) => LucideIcon::FileCode,
        Some("sh" | "zsh" | "bash" | "command") => LucideIcon::FileTerminal,
        Some("toml" | "yaml" | "yml" | "ini" | "lock" | "env" | "conf") => LucideIcon::FileCog,
        Some("json") => LucideIcon::FileJson,
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "heic" | "bmp" | "tiff") => {
            LucideIcon::FileImage
        }
        Some("mp3" | "wav" | "flac" | "aac" | "m4a" | "ogg" | "aiff") => LucideIcon::FileAudio,
        Some("mp4" | "mov" | "mkv" | "webm" | "avi") => LucideIcon::FileVideoCamera,
        Some("zip" | "tar" | "gz" | "tgz" | "rar" | "7z") => LucideIcon::FileArchive,
        Some("csv" | "xls" | "xlsx" | "numbers") => LucideIcon::FileSpreadsheet,
        Some("ttf" | "otf" | "woff" | "woff2") => LucideIcon::FileType,
        Some("md" | "txt" | "rtf" | "pdf" | "doc" | "docx" | "pages") => LucideIcon::FileText,
        _ => LucideIcon::File,
    }
}

#[cfg(test)]
mod tests {
    use super::{IconTheme, LucideIcon};

    #[test]
    fn a_name_picks_its_icon_by_its_last_extension() {
        assert_eq!(IconTheme::default().file("main.RS"), LucideIcon::FileCode);
        assert_eq!(IconTheme::default().file("lift.PNG"), LucideIcon::FileImage);
        assert_eq!(IconTheme::default().file("notes.md"), LucideIcon::FileText);
        assert_eq!(
            IconTheme::default().file("backup.tar.gz"),
            LucideIcon::FileArchive
        );
        assert_eq!(IconTheme::default().file(".env"), LucideIcon::FileCog);
        assert_eq!(
            IconTheme::default().file("take.mov"),
            LucideIcon::FileVideoCamera
        );
        assert_eq!(IconTheme::default().file("Makefile"), LucideIcon::File);
    }

    #[test]
    fn an_owners_rules_come_before_elys_map() {
        let theme = IconTheme::default()
            .name("Cargo.toml", LucideIcon::Package)
            .extension("RS", LucideIcon::Braces);
        assert_eq!(theme.file("cargo.TOML"), LucideIcon::Package);
        assert_eq!(theme.file("main.rs"), LucideIcon::Braces);
        assert_eq!(theme.file("deploy.toml"), LucideIcon::FileCog);
    }
}
