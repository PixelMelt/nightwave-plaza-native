use iced::advanced::graphics::text::cosmic_text::{fontdb, Fallback, FontSystem, PlatformFallback};
use iced::advanced::graphics::text::font_system;
use std::sync::{Arc, LazyLock};
use unicode_script::Script;

const BUNDLED: [&[u8]; 3] = [
    include_bytes!("assets/fonts/subset-Tahoma.ttf"),
    include_bytes!("assets/fonts/subset-Tahoma-Bold.ttf"),
    include_bytes!("assets/fonts/icons.ttf"),
];

const CJK_FAMILIES: &[&str] = &[
    "Noto Sans CJK JP",
    "Source Han Sans JP",
    "Droid Sans Fallback",
    "WenQuanYi Micro Hei",
    "Yu Gothic UI",
    "Meiryo",
    "MS Gothic",
    "Microsoft YaHei",
    "Hiragino Sans",
    "PingFang SC",
];

static COMMON: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut list = PlatformFallback.common_fallback().to_vec();
    list.extend(CJK_FAMILIES);
    list
});

struct CjkFallback;

impl Fallback for CjkFallback {
    fn common_fallback(&self) -> &[&'static str] {
        &COMMON
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        PlatformFallback.forbidden_fallback()
    }

    fn script_fallback(&self, script: Script, locale: &str) -> &[&'static str] {
        PlatformFallback.script_fallback(script, locale)
    }
}

fn keep(face: &fontdb::FaceInfo) -> bool {
    if !cfg!(target_os = "linux") || matches!(face.source, fontdb::Source::Binary(_)) {
        return true;
    }
    face.style == fontdb::Style::Normal
        && matches!(face.weight, fontdb::Weight::NORMAL | fontdb::Weight::BOLD)
        && face
            .families
            .iter()
            .any(|(name, _)| name.starts_with("Noto Sans") || COMMON.contains(&name.as_str()))
}

pub fn install() {
    let mut system = font_system().write().expect("font system lock");
    let raw = system.raw();
    let empty = FontSystem::new_with_locale_and_db(String::new(), fontdb::Database::new());
    let (locale, full) = std::mem::replace(raw, empty).into_locale_and_db();
    let mut db = fontdb::Database::new();
    for face in full.faces().filter(|f| keep(f)) {
        db.push_face_info(face.clone());
    }
    for bytes in BUNDLED {
        db.load_font_source(fontdb::Source::Binary(Arc::new(bytes)));
    }
    *raw = FontSystem::new_with_locale_and_db_and_fallback(locale, db, CjkFallback);
}
