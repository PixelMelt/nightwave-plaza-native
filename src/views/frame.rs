use super::bevel::bevel_button;
use super::pixel;
use super::widgets::{BOLD, Png, bevel_frame};
use crate::message::Msg;
use crate::theme;
use crate::window::WindowKind;
use iced::widget::text::{LineHeight, Shaping};
use iced::widget::{Row, Space, column, container, mouse_area, text};
use iced::window::Id;
use iced::{Element, Fill, Pixels};

static BALL: Png = Png::new(include_bytes!("../assets/icons/ball.png"));
static HELP: Png = Png::new(include_bytes!("../assets/icons/help_question_mark.png"));
static CALENDAR: Png = Png::new(include_bytes!("../assets/icons/calendar.png"));
static CHART: Png = Png::new(include_bytes!("../assets/icons/chart.png"));
static SMILEY: Png = Png::new(include_bytes!("../assets/icons/smiley.png"));
static CD: Png = Png::new(include_bytes!("../assets/icons/cd_audio.png"));
static KEYS: Png = Png::new(include_bytes!("../assets/icons/keys.png"));
static USER: Png = Png::new(include_bytes!("../assets/icons/user_computer.png"));
static INFO: Png = Png::new(include_bytes!("../assets/icons/msg_information.png"));
static DOC: Png = Png::new(include_bytes!("../assets/icons/document.png"));
static WORLD_STAR: Png = Png::new(include_bytes!("../assets/icons/world_star.png"));
static CLOCK: Png = Png::new(include_bytes!("../assets/icons/clock.png"));
static RECYCLE: Png = Png::new(include_bytes!("../assets/icons/recycle_bin_full.png"));
static GEAR: Png = Png::new(include_bytes!("../assets/icons/settings_gear.png"));

fn icon(kind: Option<WindowKind>) -> &'static Png {
    match kind {
        None | Some(WindowKind::UserRegister) => &BALL,
        Some(WindowKind::About) => &HELP,
        Some(WindowKind::History) => &CALENDAR,
        Some(WindowKind::Ratings) => &CHART,
        Some(WindowKind::Support) => &SMILEY,
        Some(WindowKind::SongInfo) => &CD,
        Some(WindowKind::UserLogin | WindowKind::UserPassword) => &KEYS,
        Some(WindowKind::UserProfile) => &USER,
        Some(WindowKind::Credits) => &INFO,
        Some(WindowKind::News) => &DOC,
        Some(WindowKind::UserFavorites | WindowKind::UserFavoritesExport) => &WORLD_STAR,
        Some(WindowKind::UserProfileEdit | WindowKind::Settings) => &GEAR,
        Some(WindowKind::UserProfileDelete) => &RECYCLE,
        Some(WindowKind::PlayerTimer) => &CLOCK,
    }
}

fn title_bar(wid: Id, kind: Option<WindowKind>, focused: bool) -> Element<'static, Msg> {
    let title = kind.map_or("Nightwave Plaza", WindowKind::title);
    let drag_area = mouse_area(
        Row::new()
            .push(icon(kind).image().width(16).height(16))
            .push(Space::new().width(2))
            .push(
                text(title)
                    .size(12)
                    .line_height(LineHeight::Absolute(Pixels(16.0)))
                    .font(BOLD)
                    .shaping(Shaping::Advanced),
            )
            .align_y(iced::Alignment::Center)
            .width(Fill)
            .height(16),
    )
    .on_press(Msg::DragWindow(wid));

    let title_button = |glyph: pixel::Pixel, msg| {
        bevel_button(container(glyph).center_x(Fill).center_y(Fill))
            .on_press(msg)
            .padding(1)
            .width(16)
            .height(16)
    };
    let buttons = Row::new()
        .push(title_button(
            pixel::minimize_glyph(),
            Msg::MinimizeWindow(wid),
        ))
        .push(title_button(pixel::close_glyph(), Msg::CloseWindow(wid)))
        .align_y(iced::Alignment::Center)
        .height(16);

    let bar = Row::new()
        .push(drag_area)
        .push(buttons)
        .push(Space::new().width(1))
        .align_y(iced::Alignment::Center)
        .height(16);

    container(bar)
        .style(if focused {
            theme::title_bar_bg
        } else {
            theme::title_bar_bg_inactive
        })
        .padding(2)
        .width(Fill)
        .into()
}

pub fn frame(
    wid: Id,
    kind: Option<WindowKind>,
    focused: bool,
    content: Element<'_, Msg>,
) -> Element<'_, Msg> {
    let framed = column![title_bar(wid, kind, focused), content]
        .spacing(1)
        .padding(1)
        .width(Fill)
        .height(Fill);
    let inner = container(framed)
        .padding(2)
        .width(Fill)
        .height(Fill)
        .style(theme::panel);
    bevel_frame(inner, theme::BEVEL_WINDOW)
        .width(Fill)
        .height(Fill)
        .into()
}
