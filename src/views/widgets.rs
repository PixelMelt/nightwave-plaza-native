use super::bevel::{Bevel, bevel_button, menu_item};
use super::pixel;
use crate::message::{Msg, PageMsg, SongInfoMsg};
use crate::state::Pager;
use crate::theme;
use chrono::{Local, TimeZone};
use iced::widget::text::{IntoFragment, LineHeight, Shaping};
use iced::widget::{
    Column, Container, Row, Space, Text, TextInput, column, container, image, mouse_area, row,
    scrollable, stack, text, text_input,
};
use iced::window::Id;
use iced::{Alignment, Element, Fill, Font, Length, Padding, Pixels};
use std::sync::OnceLock;

pub const BOLD: Font = Font {
    weight: iced::font::Weight::Bold,
    ..Font::DEFAULT
};

const ICON_FONT: Font = Font::with_name("icons");

pub const IC_CLOCK: &str = "\u{e94e}";
pub const IC_FAVORITE: &str = "\u{e9d9}";
const IC_LIKE: &str = "\u{e9da}";
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
        let handle = self
            .handle
            .get_or_init(|| image::Handle::from_bytes(self.bytes));
        image(handle.clone())
    }
}

static STATUS_GRIP: Png = Png::new(include_bytes!("../assets/icons/statusbar.png"));
static LOADING: Png = Png::new(include_bytes!("../assets/img/loading.png"));

pub fn shaped<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    text(content).shaping(Shaping::Advanced)
}

pub fn icon(glyph: &str) -> Text<'_> {
    text(glyph).font(ICON_FONT).shaping(Shaping::Advanced)
}

pub fn icon_like<'a>() -> Text<'a> {
    icon(IC_LIKE).color(theme::HEART_RED)
}

pub fn button(label: &str, width: impl Into<Length>) -> Bevel<'_, Msg> {
    let width = width.into();
    let mut label = text(label).size(11);
    if width != Length::Shrink {
        label = label.center().width(width);
    }
    bevel_button(label).width(width)
}

pub fn close_button<'a>(wid: Id) -> Element<'a, Msg> {
    button("Close", 80)
        .on_press(Msg::CloseWindow(wid))
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
        .on_press_maybe((!loading).then_some(msg))
        .width(width)
        .into()
}

pub fn link_button(label: &str, size: impl Into<Pixels>, msg: Option<Msg>) -> Element<'_, Msg> {
    let link = iced::widget::button(
        text(label)
            .size(size)
            .color(theme::LINK_COLOR)
            .line_height(LineHeight::Relative(1.5)),
    )
    .style(|_, _| theme::flat_button(theme::LINK_COLOR))
    .padding(0)
    .width(Fill);
    match msg {
        Some(msg) => mouse_area(link)
            .interaction(iced::mouse::Interaction::Pointer)
            .on_press(msg)
            .into(),
        None => link.into(),
    }
}

pub fn form_input<'a>(value: &'a str, on_input: impl Fn(String) -> Msg + 'a) -> TextInput<'a, Msg> {
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
    row![
        container(text(label).size(11)).width(label_width),
        field.into()
    ]
    .align_y(Alignment::Center)
    .into()
}

pub fn action_close_row<'a>(action: impl Into<Element<'a, Msg>>, wid: Id) -> Element<'a, Msg> {
    row![
        container(action).width(Length::FillPortion(3)),
        Space::new().width(8),
        container(button("Close", Fill).on_press(Msg::CloseWindow(wid)))
            .width(Length::FillPortion(2)),
    ]
    .into()
}

const TOP_LEFT_EDGE: Padding = Padding {
    top: 1.0,
    right: 0.0,
    bottom: 0.0,
    left: 1.0,
};
const BOTTOM_RIGHT_EDGE: Padding = Padding {
    top: 0.0,
    right: 1.0,
    bottom: 1.0,
    left: 0.0,
};

fn edge<'a>(
    content: impl Into<Element<'a, Msg>>,
    color: iced::Color,
    sides: Padding,
) -> Container<'a, Msg> {
    container(content).style(theme::fill(color)).padding(sides)
}

pub fn bevel_frame<'a>(
    content: impl Into<Element<'a, Msg>>,
    colors: theme::BevelColors,
) -> Container<'a, Msg> {
    let inner = edge(content, colors.br_inner, BOTTOM_RIGHT_EDGE);
    let inner = edge(inner, colors.tl_inner, TOP_LEFT_EDGE);
    let outer = edge(inner, colors.br_outer, BOTTOM_RIGHT_EDGE);
    edge(outer, colors.tl_outer, TOP_LEFT_EDGE)
}

pub fn sunken_frame<'a>(content: impl Into<Element<'a, Msg>>) -> Container<'a, Msg> {
    bevel_frame(content, theme::BEVEL_SUNKEN)
}

pub fn thin_sunken_frame<'a>(content: impl Into<Element<'a, Msg>>) -> Container<'a, Msg> {
    let (top_left, bottom_right) = theme::THIN_SUNKEN;
    edge(
        edge(content, bottom_right, BOTTOM_RIGHT_EDGE),
        top_left,
        TOP_LEFT_EDGE,
    )
}

pub fn sunken_panel<'a>(content: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    sunken_frame(
        container(content)
            .style(theme::panel)
            .width(Fill)
            .padding(8),
    )
    .into()
}

fn menu_bar_item(label: &str, msg: Msg) -> Element<'_, Msg> {
    let mut chars = label.chars();
    let first = chars.next().map(String::from).unwrap_or_default();
    let underlined = column![
        text(first).size(11),
        container(Space::new().width(7).height(1)).style(theme::fill(theme::BLACK)),
    ];
    menu_item(row![underlined, text(chars.as_str()).size(11)])
        .on_press(msg)
        .into()
}

pub fn menu_bar(items: impl IntoIterator<Item = (&'static str, Msg)>) -> Element<'static, Msg> {
    Row::with_children(
        items
            .into_iter()
            .map(|(label, msg)| menu_bar_item(label, msg)),
    )
    .padding(1)
    .into()
}

pub fn divider<'a>() -> Element<'a, Msg> {
    pixel::dashed_line(theme::DIVIDER_GRAY).into()
}

pub fn separator<'a>() -> Element<'a, Msg> {
    container(Space::new().width(Fill).height(1))
        .style(theme::separator)
        .into()
}

pub fn format_duration(secs: f64) -> String {
    format!("{:02}:{:02}", (secs / 60.0) as u32, (secs % 60.0) as u32)
}

fn format_timestamp(unix: u64, format: &str, fallback: &str) -> String {
    let local = i64::try_from(unix)
        .ok()
        .and_then(|secs| Local.timestamp_opt(secs, 0).single());
    match local {
        Some(time) => time.format(format).to_string(),
        None => fallback.into(),
    }
}

pub fn format_day(unix: u64) -> String {
    format_timestamp(unix, "%b %d", "???")
}

pub fn format_time_of_day(unix: u64) -> String {
    format_timestamp(unix, "%H:%M", "??:??")
}

pub fn format_date(unix: u64) -> String {
    format_timestamp(unix, "%b %d, %Y", "???")
}

#[derive(Clone, Copy)]
pub enum CellWidth {
    Shrink,
    Portion(u16),
}

pub fn status_bar(cells: Vec<(Element<'_, Msg>, CellWidth)>) -> Element<'_, Msg> {
    let cells = cells.into_iter().map(|(content, width)| {
        let cell = thin_sunken_frame(
            container(content)
                .style(theme::panel)
                .padding([3, 4])
                .width(Fill),
        );
        match width {
            CellWidth::Shrink => cell.into(),
            CellWidth::Portion(portion) => cell.width(Length::FillPortion(portion)).into(),
        }
    });
    let cells = Row::with_children(cells).spacing(2).width(Fill);
    let grip = container(STATUS_GRIP.image().width(12).height(16))
        .width(Fill)
        .height(Fill)
        .align_x(Alignment::End)
        .align_y(Alignment::End);
    container(stack![cells, grip])
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

fn centered_well<'a>(content: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    sunken_frame(
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
    centered_well(LOADING.image().width(36).height(36))
}

pub fn empty_panel(label: &str) -> Element<'_, Msg> {
    centered_well(text(label).size(11))
}

pub fn scroll_panel<'a>(content: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    sunken_frame(
        container(scrollable(content).height(Fill).style(theme::scrollbar))
            .style(theme::sunken_inner)
            .width(Fill)
            .height(Fill),
    )
    .width(Fill)
    .height(Fill)
    .into()
}

pub fn group_box<'a>(label: &'a str, body: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    let caption = container(text(format!(" {} ", label.trim())).size(11).font(BOLD))
        .style(theme::panel)
        .padding([0, 4]);
    column![caption, sunken_panel(body)].into()
}

pub fn labeled_panel<'a>(label: &'a str, field: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    sunken_panel(column![text(label).size(11), field.into()].spacing(2))
}

pub fn song_row<'a>(content: impl Into<Element<'a, Msg>>, song_id: &str) -> Element<'a, Msg> {
    let content = content.into();
    if song_id.is_empty() {
        return content;
    }
    iced::widget::button(content)
        .on_press(Msg::SongInfo(SongInfoMsg::Open(song_id.into())))
        .style(theme::list_row_button)
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
    let arrow = |glyph, on_press| {
        bevel_button(icon(glyph).size(13).center().width(Fill))
            .on_press_maybe(on_press)
            .width(33)
            .padding([1, 0])
    };
    let page_field = text_input("", &pager.input)
        .on_input(move |s| msg(PageMsg::Input(s)))
        .on_submit(submit)
        .width(34)
        .size(11)
        .style(theme::page_input)
        .padding(2);

    row![
        arrow(IC_LEFT_HAND, prev),
        sunken_frame(page_field),
        arrow(IC_RIGHT_HAND, next),
    ]
    .align_y(Alignment::Center)
    .width(100)
    .into()
}

pub fn pager_status<'a>(pager: &Pager) -> Element<'a, Msg> {
    status_bar(vec![
        (
            text(format!("Pages: {}", pager.pages)).size(10).into(),
            CellWidth::Shrink,
        ),
        (
            text(format!("Songs: {}", pager.total)).size(10).into(),
            CellWidth::Portion(1),
        ),
    ])
}

pub fn paged_footer<'a>(
    wid: Id,
    pager: &'a Pager,
    msg: impl Fn(PageMsg) -> Msg + 'a,
) -> Element<'a, Msg> {
    let controls = row![
        paginate(pager, msg),
        Space::new().width(Fill),
        close_button(wid),
    ]
    .align_y(Alignment::Center)
    .padding([4, 0]);
    column![controls, Space::new().height(2), pager_status(pager)].into()
}
