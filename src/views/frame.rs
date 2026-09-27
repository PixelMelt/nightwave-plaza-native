use crate::state::{Msg, WinType};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::pixel;
use crate::views::{bevel_2x, Png, BOLD};
use iced::widget::text::{LineHeight, Shaping};
use iced::widget::{column, container, mouse_area, text, Row, Space};
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

fn icon(wt: Option<WinType>) -> &'static Png {
    match wt {
        None | Some(WinType::UserRegister) => &BALL,
        Some(WinType::About) => &HELP,
        Some(WinType::History) => &CALENDAR,
        Some(WinType::Ratings) => &CHART,
        Some(WinType::Support) => &SMILEY,
        Some(WinType::SongInfo) => &CD,
        Some(WinType::UserLogin | WinType::UserPassword) => &KEYS,
        Some(WinType::UserProfile) => &USER,
        Some(WinType::Credits) => &INFO,
        Some(WinType::News) => &DOC,
        Some(WinType::UserFavorites | WinType::UserFavoritesExport) => &WORLD_STAR,
        Some(WinType::UserProfileEdit | WinType::Settings) => &GEAR,
        Some(WinType::UserProfileDelete) => &RECYCLE,
        Some(WinType::PlayerTimer) => &CLOCK,
    }
}

fn title_bar(wid: Id, wt: Option<WinType>, active: bool) -> Element<'static, Msg> {
    let title = wt.map_or("Nightwave Plaza", WinType::title);
    let drag_area = mouse_area(
        Row::new()
            .push(icon(wt).image().width(16).height(16))
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
    .on_press(Msg::DragWin(wid));

    let title_button = |glyph: pixel::Pixel, msg| {
        bevel_button(container(glyph).center_x(Fill).center_y(Fill))
            .on_press(msg)
            .padding(1)
            .width(16)
            .height(16)
    };
    let buttons = Row::new()
        .push(title_button(pixel::minimize_glyph(), Msg::MinimizeWin(wid)))
        .push(title_button(pixel::close_glyph(), Msg::CloseWin(wid)))
        .align_y(iced::Alignment::Center)
        .height(16);

    let bar = Row::new()
        .push(drag_area)
        .push(buttons)
        .push(Space::new().width(1))
        .align_y(iced::Alignment::Center)
        .height(16);

    container(bar)
        .style(if active {
            theme::title_bar_bg
        } else {
            theme::title_bar_bg_inactive
        })
        .padding(2)
        .width(Fill)
        .into()
}

pub fn frame<'a>(
    wid: Id,
    wt: Option<WinType>,
    active: bool,
    content: Element<'a, Msg>,
) -> Element<'a, Msg> {
    let framed = column![title_bar(wid, wt, active), content]
        .spacing(1)
        .padding(1)
        .width(Fill)
        .height(Fill);
    let inner = container(framed)
        .padding(2)
        .width(Fill)
        .height(Fill)
        .style(theme::panel);
    bevel_2x(inner, theme::BEVEL_WINDOW)
        .width(Fill)
        .height(Fill)
        .into()
}
