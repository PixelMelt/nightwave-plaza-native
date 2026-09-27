use crate::state::{Msg, PageMsg, Pager, SongInfoMsg};
use crate::theme;
use crate::views::bevel::{bevel_button, menu_item, Bevel};
use crate::views::pixel;
use chrono::{Local, TimeZone};
use iced::widget::text::{LineHeight, Shaping};
use iced::widget::{container, image, mouse_area, text, text_input, Column, Row, Space};
use iced::window::Id;
use iced::{Element, Fill, Font, Length, Padding};
use std::sync::OnceLock;

pub const BOLD: Font = Font {
    weight: iced::font::Weight::Bold,
    ..Font::DEFAULT
};

pub const ICON_FONT: Font = Font {
    family: iced::font::Family::Name("icons"),
    ..Font::DEFAULT
};

pub const IC_CLOCK: &str = "\u{e94e}";
pub const IC_FAVORITE: &str = "\u{e9d9}";
pub const IC_LIKE: &str = "\u{e9da}";
const IC_RIGHT_HAND: &str = "\u{ea42}";
const IC_LEFT_HAND: &str = "\u{ea44}";

pub struct Png {
    bytes: &'static [u8],
    handle: OnceLock<image::Handle>,
}

impl Png {
    pub const fn new(bytes: &'static [u8]) -> Self {
        Self {
            bytes,
            handle: OnceLock::new(),
        }
    }

    pub fn image(&self) -> image::Image {
        image(
            self.handle
                .get_or_init(|| image::Handle::from_bytes(self.bytes))
                .clone(),
        )
    }
}

static STATUS_GRIP: Png = Png::new(include_bytes!("../assets/icons/statusbar.png"));
static LOADING: Png = Png::new(include_bytes!("../assets/img/loading.png"));

pub fn shaped<'a>(s: impl ToString) -> iced::widget::Text<'a> {
    text(s.to_string()).shaping(Shaping::Advanced)
}

pub fn icon<'a>(glyph: &'a str) -> iced::widget::Text<'a> {
    text(glyph).font(ICON_FONT).shaping(Shaping::Advanced)
}

pub fn icon_like<'a>() -> iced::widget::Text<'a> {
    icon(IC_LIKE).color(theme::HEART_RED)
}

pub fn button<'a>(label: &'a str, width: impl Into<Length>) -> Bevel<'a, Msg> {
    let width = width.into();
    let mut label = text(label).size(11);
    if width != Length::Shrink {
        label = label.center().width(width);
    }
    bevel_button(label).width(width)
}

pub fn close_btn<'a>(wid: Id) -> Element<'a, Msg> {
    button("Close", 80)
        .on_press(Msg::CloseWin(wid))
        .padding([4, 0])
        .into()
}

pub fn submit_button<'a>(
    loading: bool,
    busy_label: &'a str,
    idle_label: &'a str,
    msg: Msg,
    width: impl Into<Length>,
) -> Element<'a, Msg> {
    let width = width.into();
    let label = if loading { busy_label } else { idle_label };
    bevel_button(text(label).size(11).font(BOLD).center().width(width))
        .maybe_on_press((!loading).then_some(msg))
        .width(width)
        .into()
}

pub fn link_button<'a>(
    label: &'a str,
    size: impl Into<iced::Pixels>,
    msg: Option<Msg>,
) -> Element<'a, Msg> {
    let btn = iced::widget::button(
        text(label)
            .size(size)
            .color(theme::LINK_COLOR)
            .line_height(LineHeight::Relative(1.5)),
    )
    .style(|_, _| theme::flat_button(theme::LINK_COLOR))
    .padding(0)
    .width(Fill);
    match msg {
        Some(msg) => mouse_area(btn)
            .interaction(iced::mouse::Interaction::Pointer)
            .on_press(msg)
            .into(),
        None => btn.into(),
    }
}

pub fn form_error(error: &Option<String>) -> Element<'_, Msg> {
    match error {
        Some(err) => text(err).size(11).color(theme::ERROR_RED).into(),
        None => Space::new().height(0).into(),
    }
}

pub fn form_input<'a>(
    value: &'a str,
    on_input: impl Fn(String) -> Msg + 'a,
) -> iced::widget::TextInput<'a, Msg> {
    text_input("", value)
        .on_input(on_input)
        .size(11)
        .padding([3, 4])
        .style(theme::page_input)
}

pub fn form_field_row<'a>(
    label: &'a str,
    label_width: impl Into<Length>,
    field: impl Into<Element<'a, Msg>>,
) -> Element<'a, Msg> {
    Row::new()
        .push(container(text(label).size(11)).width(label_width))
        .push(field.into())
        .align_y(iced::Alignment::Center)
        .into()
}

pub fn action_close_row<'a>(action: impl Into<Element<'a, Msg>>, wid: Id) -> Element<'a, Msg> {
    Row::new()
        .push(container(action).width(Length::FillPortion(3)))
        .push(Space::new().width(8))
        .push(
            container(button("Close", Fill).on_press(Msg::CloseWin(wid)))
                .width(Length::FillPortion(2)),
        )
        .into()
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

fn bevel_layer<'a>(
    content: impl Into<Element<'a, Msg>>,
    color: iced::Color,
    padding: Padding,
) -> iced::widget::Container<'a, Msg> {
    container(content)
        .style(theme::fill(color))
        .padding(padding)
}

pub fn bevel_2x<'a>(
    content: impl Into<Element<'a, Msg>>,
    colors: theme::BevelColors,
) -> iced::widget::Container<'a, Msg> {
    let l4 = bevel_layer(content, colors.br_inner, BR);
    let l3 = bevel_layer(l4, colors.tl_inner, TL);
    let l2 = bevel_layer(l3, colors.br_outer, BR);
    bevel_layer(l2, colors.tl_outer, TL)
}

pub fn d3_sunken<'a>(content: impl Into<Element<'a, Msg>>) -> iced::widget::Container<'a, Msg> {
    bevel_2x(content, theme::BEVEL_SUNKEN)
}

pub fn d3_thin_sunken<'a>(
    content: impl Into<Element<'a, Msg>>,
) -> iced::widget::Container<'a, Msg> {
    let (tl, br) = theme::THIN_SUNKEN;
    bevel_layer(bevel_layer(content, br, BR), tl, TL)
}

pub fn sunken_panel<'a>(content: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    d3_sunken(
        container(content)
            .style(theme::panel)
            .width(Fill)
            .padding(8),
    )
    .into()
}

fn menu_btn_underline(label: &str, msg: Msg) -> Element<'_, Msg> {
    let mut chars = label.chars();
    let first = chars.next().unwrap().to_string();
    let rest = chars.as_str().to_string();

    let first_col = iced::widget::column![
        text(first).size(11),
        container(Space::new().width(7).height(1)).style(theme::fill(theme::BLACK)),
    ];
    let label_row = iced::widget::row![first_col, text(rest).size(11)];
    menu_item(label_row).on_press(msg).into()
}

pub fn menu_bar(items: impl IntoIterator<Item = (&'static str, Msg)>) -> Element<'static, Msg> {
    let mut row = Row::new();
    for (label, msg) in items {
        row = row.push(menu_btn_underline(label, msg));
    }
    row.padding(1).into()
}

pub fn divider<'a>() -> Element<'a, Msg> {
    pixel::dashed_line(theme::DIVIDER_GRAY).into()
}

pub fn separator<'a>() -> Element<'a, Msg> {
    container(Space::new().width(Fill).height(1))
        .style(theme::separator)
        .into()
}

pub fn format_time(secs: f64) -> String {
    format!("{:02}:{:02}", (secs / 60.0) as u32, (secs % 60.0) as u32)
}

fn fmt_ts(ts: u64, fmt: &str, fallback: &str) -> String {
    match Local.timestamp_opt(ts as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format(fmt).to_string(),
        _ => fallback.to_string(),
    }
}

pub fn format_timestamp_day(ts: u64) -> String {
    fmt_ts(ts, "%b %d", "???")
}

pub fn format_timestamp_time(ts: u64) -> String {
    fmt_ts(ts, "%H:%M", "??:??")
}

pub fn format_date(ts: u64) -> String {
    fmt_ts(ts, "%b %d, %Y", "???")
}

pub fn status_bar<'a>(cells: Vec<(Element<'a, Msg>, u16)>) -> Element<'a, Msg> {
    let mut r = Row::new().spacing(2).width(Fill);
    for (cell, portion) in cells {
        let inner = container(cell)
            .style(theme::panel)
            .padding([3, 4])
            .width(Fill);
        let boxed = d3_thin_sunken(inner);
        if portion > 0 {
            r = r.push(boxed.width(Length::FillPortion(portion)));
        } else {
            r = r.push(boxed);
        }
    }
    let grip = container(STATUS_GRIP.image().width(12).height(16))
        .width(Fill)
        .height(Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom);
    container(iced::widget::stack![r, grip])
        .style(theme::panel)
        .width(Fill)
        .padding(Padding {
            top: 2.0,
            right: 1.0,
            bottom: 1.0,
            left: 1.0,
        })
        .into()
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

pub fn loading_panel<'a>() -> Element<'a, Msg> {
    fill_sunken(LOADING.image().width(36).height(36))
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
    let chip = container(text(format!(" {} ", label.trim())).size(11).font(BOLD))
        .style(theme::panel)
        .padding([0, 4]);
    Column::new().push(chip).push(sunken_panel(body)).into()
}

pub fn labeled_panel<'a>(label: &'a str, field: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    sunken_panel(
        Column::new()
            .push(text(label).size(11))
            .push(field.into())
            .spacing(2),
    )
}

pub fn clickable_row<'a>(content: impl Into<Element<'a, Msg>>, song_id: &str) -> Element<'a, Msg> {
    let content = content.into();
    if song_id.is_empty() {
        return content;
    }
    iced::widget::button(content)
        .on_press(Msg::SongInfo(SongInfoMsg::Open(song_id.to_string())))
        .style(theme::list_row_btn)
        .padding(0)
        .width(Fill)
        .into()
}

pub fn song_list<'a, T>(
    items: &'a [T],
    mut render: impl FnMut(usize, &'a T) -> Element<'a, Msg>,
) -> Element<'a, Msg> {
    let mut list = Column::new().width(Fill);
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            list = list.push(divider());
        }
        list = list.push(render(i, item));
    }
    scroll_panel(list)
}

pub fn paginate<'a>(pager: &'a Pager, msg: impl Fn(PageMsg) -> Msg + 'a) -> Element<'a, Msg> {
    let (page, pages) = (pager.page, pager.pages);
    if pages <= 1 {
        return Space::new().width(0).into();
    }
    let prev = (page > 1 && !pager.loading).then(|| msg(PageMsg::Go(page - 1)));
    let next = (page < pages && !pager.loading).then(|| msg(PageMsg::Go(page + 1)));
    let submit = msg(PageMsg::Submit);
    let hand = |glyph| icon(glyph).size(13).center().width(Fill);

    Row::new()
        .align_y(iced::Alignment::Center)
        .width(100)
        .push(
            bevel_button(hand(IC_LEFT_HAND))
                .maybe_on_press(prev)
                .width(33)
                .padding([1, 0]),
        )
        .push(d3_sunken(
            text_input("", &pager.input)
                .on_input(move |s| msg(PageMsg::Input(s)))
                .on_submit(submit)
                .width(34)
                .size(11)
                .style(theme::page_input)
                .padding(2),
        ))
        .push(
            bevel_button(hand(IC_RIGHT_HAND))
                .maybe_on_press(next)
                .width(33)
                .padding([1, 0]),
        )
        .into()
}

pub fn pager_status<'a>(pager: &Pager) -> Element<'a, Msg> {
    status_bar(vec![
        (text(format!("Pages: {}", pager.pages)).size(10).into(), 0),
        (text(format!("Songs: {}", pager.total)).size(10).into(), 1),
    ])
}

pub fn paged_footer<'a>(
    wid: Id,
    pager: &'a Pager,
    msg: impl Fn(PageMsg) -> Msg + 'a,
) -> Element<'a, Msg> {
    let bottom = Row::new()
        .push(paginate(pager, msg))
        .push(Space::new().width(Fill))
        .push(close_btn(wid))
        .align_y(iced::Alignment::Center)
        .padding([4, 0]);
    Column::new()
        .push(bottom)
        .push(Space::new().height(2))
        .push(pager_status(pager))
        .into()
}
