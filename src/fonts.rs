use iced::advanced::graphics::text::cosmic_text::{fontdb, Fallback, FontSystem, PlatformFallback};
use iced::advanced::graphics::text::font_system;
use std::sync::LazyLock;
use unicode_script::Script;

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

pub fn install_fallback() {
    let mut system = font_system().write().expect("font system lock");
    let raw = system.raw();
    let empty = FontSystem::new_with_locale_and_db(String::new(), fontdb::Database::new());
    let (locale, db) = std::mem::replace(raw, empty).into_locale_and_db();
    *raw = FontSystem::new_with_locale_and_db_and_fallback(locale, db, CjkFallback);
}
