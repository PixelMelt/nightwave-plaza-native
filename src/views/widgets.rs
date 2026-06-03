use crate::state::{Msg, WinType};
use crate::theme;
use crate::views::bevel::{bevel_button, menu_item, title_button};
use chrono::{Local, TimeZone};
use iced::widget::text::Shaping;
use iced::widget::{container, image, mouse_area, svg, text, text_input, Column, Row, Space};
use iced::{Color, Element, Fill, Length, Padding, Theme};

pub const ICON_FONT: iced::Font = iced::Font {
    family: iced::font::Family::Name("icons"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub fn static_image(bytes: &'static [u8]) -> image::Handle {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<usize, image::Handle>>> = OnceLock::new();
    CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .entry(bytes.as_ptr() as usize)
        .or_insert_with(|| image::Handle::from_bytes(bytes))
        .clone()
}

pub const IC_CLOCK: &str = "\u{e94e}";
pub const IC_USER: &str = "\u{e971}";
pub const IC_COG: &str = "\u{e994}";
pub const IC_FAVORITE: &str = "\u{e9d9}";
pub const IC_LIKE: &str = "\u{e9da}";
pub const IC_RIGHT_HAND: &str = "\u{ea42}";
pub const IC_LEFT_HAND: &str = "\u{ea44}";

const CLOSE_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 9 9'><line stroke='#000' stroke-width='1.5' x1='1.5' y1='1.5' x2='7.5' y2='7.5'/><line stroke='#000' stroke-width='1.5' x1='7.5' y1='1.5' x2='1.5' y2='7.5'/></svg>";
const MINIMIZE_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 9 9'><rect x='1' y='7' width='6' height='2' fill='#000'/></svg>";
const DIVIDER_SVG: &[u8] = b"<svg width='100%' height='1' xmlns='http://www.w3.org/2000/svg'><line x1='0' y1='0' x2='100%' y2='0' stroke='#c8c8c8' stroke-width='1' stroke-dasharray='2,3'/></svg>";

const WIN_BALL: &[u8] = include_bytes!("../assets/icons/ball.png");
const WIN_HELP: &[u8] = include_bytes!("../assets/icons/help_question_mark.png");
const WIN_CALENDAR: &[u8] = include_bytes!("../assets/icons/calendar.png");
const WIN_CHART: &[u8] = include_bytes!("../assets/icons/chart.png");
const WIN_SMILEY: &[u8] = include_bytes!("../assets/icons/smiley.png");
const WIN_CD: &[u8] = include_bytes!("../assets/icons/cd_audio.png");
const WIN_KEYS: &[u8] = include_bytes!("../assets/icons/keys.png");
const WIN_USER: &[u8] = include_bytes!("../assets/icons/user_computer.png");
const WIN_INFO: &[u8] = include_bytes!("../assets/icons/msg_information.png");
const WIN_DOC: &[u8] = include_bytes!("../assets/icons/document.png");
const WIN_WORLD_STAR: &[u8] = include_bytes!("../assets/icons/world_star.png");
const WIN_CLOCK: &[u8] = include_bytes!("../assets/icons/clock.png");
const WIN_RECYCLE: &[u8] = include_bytes!("../assets/icons/recycle_bin_full.png");
const WIN_GEAR: &[u8] = include_bytes!("../assets/icons/settings_gear.png");
const FAVICON: &[u8] = include_bytes!("../assets/icons/favicon-32x32.png");

pub const HEART_RED: Color = Color::from_rgb(0.757, 0.153, 0.153);
pub const LINK_COLOR: Color = Color::from_rgb(0.024, 0.271, 0.678);
pub const MUTED: Color = Color::from_rgb(0.4, 0.4, 0.4);

pub fn bold_font() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    }
}

fn bevel_layer<'a>(
    content: impl Into<Element<'a, Msg>>,
    bg: Color,
    padding: Padding,
) -> iced::widget::Container<'a, Msg> {
    container(content)
        .style(move |_: &Theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            ..Default::default()
        })
        .padding(padding)
}

const TL: Padding = Padding {
    top: 1.0,
    right: 0.0,
    bottom: 0.0,
    left: 1.0,
};
const BR: Padding = Padding {
    top: 0.0,
    right: 1.0,
    bottom: 1.0,
    left: 0.0,
};

fn bevel_2x<'a>(
    content: impl Into<Element<'a, Msg>>,
    tl_outer: Color,
    tl_inner: Color,
    br_inner: Color,
    br_outer: Color,
) -> iced::widget::Container<'a, Msg> {
    let l4 = bevel_layer(content, br_inner, BR);
    let l3 = bevel_layer(l4, tl_inner, TL);
    let l2 = bevel_layer(l3, br_outer, BR);
    bevel_layer(l2, tl_outer, TL)
}

pub fn d3_raised_window<'a>(
    content: impl Into<Element<'a, Msg>>,
) -> iced::widget::Container<'a, Msg> {
    bevel_2x(
        content,
        theme::LIGHT_GRAY,
        theme::WHITE,
        theme::DARK_GRAY,
        theme::BLACK,
    )
}

pub fn d3_sunken<'a>(content: impl Into<Element<'a, Msg>>) -> iced::widget::Container<'a, Msg> {
    bevel_2x(
        content,
        theme::DARK_GRAY,
        theme::BLACK,
        theme::LIGHT_GRAY,
        theme::WHITE,
    )
}

pub fn d3_thin_sunken<'a>(
    content: impl Into<Element<'a, Msg>>,
) -> iced::widget::Container<'a, Msg> {
    let inner = bevel_layer(content, theme::WHITE, BR);
    bevel_layer(inner, theme::DARK_GRAY, TL)
}

fn win_icon_bytes(wt: Option<&WinType>) -> &'static [u8] {
    match wt {
        Some(WinType::About) => WIN_HELP,
        Some(WinType::History) => WIN_CALENDAR,
        Some(WinType::Ratings) => WIN_CHART,
        Some(WinType::Support) => WIN_SMILEY,
        Some(WinType::SongInfo) => WIN_CD,
        Some(WinType::UserLogin) => WIN_KEYS,
        Some(WinType::UserProfile) => WIN_USER,
        Some(WinType::UserRegister) => WIN_BALL,
        Some(WinType::Credits) => WIN_INFO,
        Some(WinType::News) => WIN_DOC,
        Some(WinType::UserFavorites) => WIN_WORLD_STAR,
        Some(WinType::UserFavoritesExport) => WIN_WORLD_STAR,
        Some(WinType::UserProfileEdit) => WIN_GEAR,
        Some(WinType::UserPassword) => WIN_KEYS,
        Some(WinType::UserProfileDelete) => WIN_RECYCLE,
        Some(WinType::PlayerTimer) => WIN_CLOCK,
        Some(WinType::Settings) => WIN_GEAR,
        None => FAVICON,
    }
}

pub fn icon_like<'a>() -> iced::widget::Text<'a, Theme> {
    text(IC_LIKE)
        .font(ICON_FONT)
        .color(HEART_RED)
        .shaping(Shaping::Advanced)
}

pub fn icon_clock<'a>() -> iced::widget::Text<'a, Theme> {
    text(IC_CLOCK).font(ICON_FONT).shaping(Shaping::Advanced)
}

pub fn shaped<'a>(s: impl ToString) -> iced::widget::Text<'a, Theme> {
    text(s.to_string()).shaping(Shaping::Advanced)
}

pub fn menu_btn_underline(label: &str, msg: Msg) -> Element<'_, Msg> {
    let first: String = label.chars().next().unwrap_or(' ').to_string();
    let rest: String = if label.len() > first.len() {
        label[first.len()..].to_string()
    } else {
        String::new()
    };

    let first_col = iced::widget::column![
        text(first).size(11),
        container(Space::new().width(7).height(1)).style(move |_: &Theme| container::Style {
            background: Some(iced::Background::Color(theme::BLACK)),
            ..Default::default()
        }),
    ]
    .spacing(0);

    let label_row = iced::widget::row![first_col, text(rest).size(11)].spacing(0);

    menu_item(label_row).on_press(msg).into()
}

pub fn menu_bar<I>(items: I) -> Element<'static, Msg>
where
    I: IntoIterator<Item = (&'static str, Msg)>,
{
    let mut row = Row::new();
    for (label, msg) in items {
        row = row.push(menu_btn_underline(label, msg));
    }
    row.padding([1, 1]).into()
}

pub fn divider<'a>() -> Element<'a, Msg> {
    let svg_widget = svg(svg::Handle::from_memory(DIVIDER_SVG))
        .width(iced::Fill)
        .height(1);
    container(svg_widget)
        .width(iced::Fill)
        .height(1)
        .clip(true)
        .into()
}

pub fn format_time(secs: f64) -> String {
    let m = (secs / 60.0) as u32;
    let s = (secs % 60.0) as u32;
    format!("{:02}:{:02}", m, s)
}

pub fn format_timestamp_day(ts: u64) -> String {
    match Local.timestamp_opt(ts as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%b %d").to_string(),
        _ => "???".to_string(),
    }
}

pub fn format_timestamp_time(ts: u64) -> String {
    match Local.timestamp_opt(ts as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%H:%M").to_string(),
        _ => "??:??".to_string(),
    }
}

pub fn format_date(ts: u64) -> String {
    match Local.timestamp_opt(ts as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%b %d, %Y").to_string(),
        _ => "???".to_string(),
    }
}

pub fn pagination<'a>(
    page: u32,
    pages: u32,
    page_input: &str,
    loading: bool,
    on_input: impl Fn(String) -> Msg + 'a,
    on_submit: Msg,
    on_prev: Option<Msg>,
    on_next: Option<Msg>,
) -> Element<'a, Msg> {
    let mut r = Row::new()
        .spacing(0)
        .align_y(iced::Alignment::Center)
        .width(100);

    let left_icon = text(IC_LEFT_HAND)
        .font(ICON_FONT)
        .size(13)
        .center()
        .shaping(Shaping::Advanced)
        .width(iced::Fill);

    let prev_msg = if page > 1 && !loading { on_prev } else { None };
    r = r.push(
        bevel_button(left_icon)
            .maybe_on_press(prev_msg)
            .width(33)
            .padding([1, 0]),
    );

    r = r.push(d3_sunken(
        text_input("", page_input)
            .on_input(on_input)
            .on_submit(on_submit)
            .width(34)
            .size(11)
            .style(theme::page_input)
            .padding([2, 2]),
    ));

    let right_icon = text(IC_RIGHT_HAND)
        .font(ICON_FONT)
        .size(13)
        .center()
        .shaping(Shaping::Advanced)
        .width(iced::Fill);

    let next_msg = if page < pages && !loading {
        on_next
    } else {
        None
    };
    r = r.push(
        bevel_button(right_icon)
            .maybe_on_press(next_msg)
            .width(33)
            .padding([1, 0]),
    );

    r.into()
}

pub fn status_bar<'a>(cells: Vec<Element<'a, Msg>>) -> Element<'a, Msg> {
    let mut r = Row::new().spacing(2);
    for cell in cells {
        r = r.push(d3_thin_sunken(
            container(cell).style(theme::status_cell).padding([3, 4]),
        ));
    }
    container(r)
        .style(theme::status_bar)
        .width(iced::Fill)
        .padding([2, 1])
        .into()
}

pub fn title_bar(
    title: String,
    wid: iced::window::Id,
    wt: Option<&WinType>,
    show_minimize: bool,
    show_close: bool,
) -> Element<'static, Msg> {
    let icon_bytes = win_icon_bytes(wt);
    let icon = image(static_image(icon_bytes)).width(16).height(16);

    let title_label = text(title)
        .size(12)
        .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(16.0)))
        .font(bold_font())
        .shaping(Shaping::Advanced);

    let drag_content = Row::new()
        .push(icon)
        .push(Space::new().width(2))
        .push(title_label)
        .align_y(iced::Alignment::Center)
        .width(Fill)
        .height(16);

    let drag_area: Element<'static, Msg> =
        mouse_area(drag_content).on_press(Msg::DragWin(wid)).into();

    let mut buttons = Row::new()
        .spacing(0)
        .align_y(iced::Alignment::Center)
        .height(16);

    if show_minimize {
        let min_svg = svg(svg::Handle::from_memory(MINIMIZE_SVG))
            .width(9)
            .height(9);
        let min_content = container(min_svg).center_x(Fill).center_y(Fill);

        buttons = buttons.push(
            title_button(min_content)
                .on_press(Msg::MinimizeWin(wid))
                .width(16)
                .height(16),
        );
    }

    if show_close {
        let close_svg = svg(svg::Handle::from_memory(CLOSE_SVG)).width(9).height(9);
        let close_content = container(close_svg).center_x(Fill).center_y(Fill);

        buttons = buttons.push(
            title_button(close_content)
                .on_press(Msg::CloseWin(wid))
                .width(16)
                .height(16),
        );
    }

    let bar = Row::new()
        .push(drag_area)
        .push(buttons)
        .align_y(iced::Alignment::Center)
        .height(16);

    container(bar)
        .style(theme::title_bar_bg)
        .padding(2)
        .width(Fill)
        .into()
}

pub fn raised_btn<'a>(
    label: &'a str,
    msg: Msg,
    width: impl Into<Length>,
    padding: impl Into<Padding>,
) -> Element<'a, Msg> {
    let w = width.into();
    bevel_button(text(label).size(11).center().width(w))
        .on_press(msg)
        .width(w)
        .padding(padding)
        .into()
}

pub fn close_btn<'a>(wid: iced::window::Id) -> Element<'a, Msg> {
    raised_btn("Close", Msg::CloseWin(wid), 80.0, Padding::from([4, 0]))
}

pub fn close_btn_padded<'a>(wid: iced::window::Id) -> Element<'a, Msg> {
    raised_btn(
        "Close",
        Msg::CloseWin(wid),
        Length::Shrink,
        Padding::from([4, 24]),
    )
}

fn fill_sunken<'a>(content: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    d3_sunken(
        container(content)
            .style(theme::sunken_inner)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill),
    )
    .width(Fill)
    .height(Fill)
    .into()
}

const LOADING_IMG: &[u8] = include_bytes!("../assets/img/loading.png");

pub fn loading_panel<'a>() -> Element<'a, Msg> {
    fill_sunken(image(static_image(LOADING_IMG)).width(36).height(36))
}

pub fn empty_panel<'a>(label: &'a str) -> Element<'a, Msg> {
    fill_sunken(text(label).size(11))
}

pub fn scroll_panel<'a>(content: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    d3_sunken(
        container(
            iced::widget::scrollable(content)
                .height(Fill)
                .style(theme::scrollbar),
        )
        .style(theme::sunken_inner)
        .width(Fill)
        .height(Fill),
    )
    .width(Fill)
    .height(Fill)
    .into()
}

pub fn group_box<'a>(label: &'a str, body: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    let chip = container(
        text(format!(" {} ", label.trim()))
            .size(11)
            .font(bold_font()),
    )
    .style(theme::group_label)
    .padding([0, 4]);

    Column::new()
        .push(chip)
        .push(d3_sunken(
            container(body).style(theme::panel).width(Fill).padding(8),
        ))
        .spacing(0)
        .into()
}
