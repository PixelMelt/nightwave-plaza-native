use crate::state::{Msg, WinType};
use crate::theme;
use chrono::{Local, TimeZone};
use iced::widget::text::Shaping;
use iced::widget::{button, container, image, mouse_area, svg, text, text_input, Row, Space};
use iced::{Color, Element, Fill, Padding, Theme};

pub const ICON_FONT: iced::Font = iced::Font {
    family: iced::font::Family::Name("icons"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const IC_CLOCK: &str = "\u{e94e}";
pub const IC_USER: &str = "\u{e971}";
pub const IC_COG: &str = "\u{e994}";
pub const IC_FAVORITE: &str = "\u{e9d9}";
pub const IC_LIKE: &str = "\u{e9da}";
pub const IC_RIGHT_HAND: &str = "\u{ea42}";
pub const IC_LEFT_HAND: &str = "\u{ea44}";

const CLOSE_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 9 9'><line stroke='#000' stroke-width='1.5' x1='1.5' y1='1.5' x2='7.5' y2='7.5'/><line stroke='#000' stroke-width='1.5' x1='7.5' y1='1.5' x2='1.5' y2='7.5'/></svg>";

const MINIMIZE_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 9 9'><rect x='1' y='7' width='6' height='2' fill='#000'/></svg>";

const WIN_BALL: &[u8] = include_bytes!("../../assets/icons/win_ball.png");
const WIN_HELP: &[u8] = include_bytes!("../../assets/icons/win_help_question_mark.png");
const WIN_CALENDAR: &[u8] = include_bytes!("../../assets/icons/win_calendar.png");
const WIN_CHART: &[u8] = include_bytes!("../../assets/icons/win_chart.png");
const WIN_SMILEY: &[u8] = include_bytes!("../../assets/icons/win_smiley.png");
const WIN_CD: &[u8] = include_bytes!("../../assets/icons/win_cd_audio.png");
const WIN_KEYS: &[u8] = include_bytes!("../../assets/icons/win_keys.png");
const WIN_USER: &[u8] = include_bytes!("../../assets/icons/win_user_computer.png");
const WIN_INFO: &[u8] = include_bytes!("../../assets/icons/win_msg_information.png");
const WIN_DOC: &[u8] = include_bytes!("../../assets/icons/win_document.png");
const FAVICON: &[u8] = include_bytes!("../../assets/icons/favicon-32x32.png");

fn d3_border<'a>(
    content: impl Into<Element<'a, Msg>>,
    highlight: Color,
    shadow: Color,
) -> iced::widget::Container<'a, Msg> {
    let inner = container(content)
        .style(move |_: &Theme| container::Style {
            background: Some(iced::Background::Color(highlight)),
            ..Default::default()
        })
        .padding(Padding {
            top: 1.0,
            right: 0.0,
            bottom: 0.0,
            left: 1.0,
        });

    container(inner)
        .style(move |_: &Theme| container::Style {
            background: Some(iced::Background::Color(shadow)),
            ..Default::default()
        })
        .padding(Padding {
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
            left: 0.0,
        })
}

pub fn d3_raised<'a>(content: impl Into<Element<'a, Msg>>) -> iced::widget::Container<'a, Msg> {
    d3_border(content, theme::WHITE, theme::BLACK)
}

pub fn d3_raised_window<'a>(
    content: impl Into<Element<'a, Msg>>,
) -> iced::widget::Container<'a, Msg> {
    d3_border(content, theme::LIGHT_GRAY, theme::BLACK)
}

pub fn d3_sunken<'a>(content: impl Into<Element<'a, Msg>>) -> iced::widget::Container<'a, Msg> {
    d3_border(content, theme::DARK_GRAY, theme::WHITE)
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
        None => FAVICON,
    }
}

pub const HEART_RED: Color = Color::from_rgb(0.757, 0.153, 0.153);

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

pub fn menu_btn_underline(label: &str, msg: Msg) -> Element<Msg> {
    let first: String = label.chars().next().unwrap_or(' ').to_string();
    let rest: String = if label.len() > first.len() {
        label[first.len()..].to_string()
    } else {
        String::new()
    };

    let first_col = iced::widget::column![
        text(first).size(11),
        container(Space::new(7, 1)).style(move |_: &Theme| container::Style {
            background: Some(iced::Background::Color(theme::BLACK)),
            ..Default::default()
        }),
    ]
    .spacing(0);

    let label_row = iced::widget::row![first_col, text(rest).size(11)].spacing(0);

    button(label_row)
        .on_press(msg)
        .style(theme::menu_btn)
        .padding([3, 6])
        .into()
}

pub const DIVIDER_COLOR: Color = Color::from_rgb(0.784, 0.784, 0.784);

pub fn divider<'a>() -> Element<'a, Msg> {
    let mut dots = iced::widget::Row::new().height(1);
    for _ in 0..88 {
        dots = dots.push(container(Space::new(2, 1)).style(move |_: &Theme| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(DIVIDER_COLOR)),
                ..Default::default()
            }
        }));
        dots = dots.push(Space::new(3, 1));
    }
    container(dots)
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

    if page > 1 && !loading {
        if let Some(prev_msg) = on_prev {
            r = r.push(d3_raised(
                button(left_icon)
                    .on_press(prev_msg)
                    .style(theme::raised)
                    .width(33)
                    .padding([1, 0]),
            ));
        } else {
            r = r.push(d3_raised(
                button(left_icon)
                    .style(theme::raised)
                    .width(33)
                    .padding([1, 0]),
            ));
        }
    } else {
        r = r.push(d3_raised(
            button(left_icon)
                .style(theme::raised)
                .width(33)
                .padding([1, 0]),
        ));
    }

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

    if page < pages && !loading {
        if let Some(next_msg) = on_next {
            r = r.push(d3_raised(
                button(right_icon)
                    .on_press(next_msg)
                    .style(theme::raised)
                    .width(33)
                    .padding([1, 0]),
            ));
        } else {
            r = r.push(d3_raised(
                button(right_icon)
                    .style(theme::raised)
                    .width(33)
                    .padding([1, 0]),
            ));
        }
    } else {
        r = r.push(d3_raised(
            button(right_icon)
                .style(theme::raised)
                .width(33)
                .padding([1, 0]),
        ));
    }

    r.into()
}

pub fn status_bar<'a>(cells: Vec<Element<'a, Msg>>) -> Element<'a, Msg> {
    let mut r = Row::new().spacing(2);
    for cell in cells {
        r = r.push(d3_sunken(
            container(cell).style(theme::status_cell).padding([2, 4]),
        ));
    }
    container(r)
        .style(theme::status_bar)
        .width(iced::Fill)
        .padding([1, 1])
        .into()
}

pub fn title_bar(
    title: String,
    wid: iced::window::Id,
    wt: Option<&WinType>,
    show_minimize: bool,
    show_close: bool,
) -> Element<'static, Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };

    let icon_bytes = win_icon_bytes(wt);
    let icon = image(image::Handle::from_bytes(icon_bytes))
        .width(16)
        .height(16);

    let title_label = text(title).size(12).font(bold).shaping(Shaping::Advanced);

    let drag_content = Row::new()
        .push(icon)
        .push(Space::with_width(2))
        .push(title_label)
        .align_y(iced::Alignment::Center)
        .width(Fill)
        .height(16);

    let drag_area: Element<'static, Msg> =
        mouse_area(drag_content).on_press(Msg::DragWin(wid)).into();

    let mut bar = Row::new()
        .push(drag_area)
        .spacing(2)
        .align_y(iced::Alignment::Center)
        .height(16);

    if show_minimize {
        let min_svg = svg(svg::Handle::from_memory(MINIMIZE_SVG))
            .width(9)
            .height(9);
        let min_content = container(min_svg).center_x(Fill).center_y(Fill);

        bar = bar.push(d3_raised(
            button(min_content)
                .on_press(Msg::MinimizeWin(wid))
                .style(theme::raised)
                .padding(2)
                .width(16)
                .height(16),
        ));
    }

    if show_close {
        let close_svg = svg(svg::Handle::from_memory(CLOSE_SVG)).width(9).height(9);
        let close_content = container(close_svg).center_x(Fill).center_y(Fill);

        bar = bar.push(d3_raised(
            button(close_content)
                .on_press(Msg::CloseWin(wid))
                .style(theme::raised)
                .padding(2)
                .width(16)
                .height(16),
        ));
    }

    container(bar)
        .style(theme::title_bar_bg)
        .padding(2)
        .width(Fill)
        .into()
}
