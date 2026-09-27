use super::widgets::{BOLD, Png, close_button, link_button, sunken_frame};
use crate::message::Msg;
use crate::theme;
use iced::widget::{Space, column, container, mouse_area, row, text};
use iced::window::Id;
use iced::{Element, Fill};

static BOOSTY: Png = Png::new(include_bytes!("../assets/img/boosty.png"));
const BOOSTY_URL: &str = "https://boosty.to/nightwaveplaza";

pub fn view(wid: Id) -> Element<'static, Msg> {
    let title = text("Love Nightwave Plaza?")
        .size(14)
        .font(BOLD)
        .center()
        .width(Fill);

    let info = column![
        text("Support the radio station and future updates by donating via Boosty to receive special Discord rewards!")
            .size(11).center().width(Fill),
        Space::new().height(4),
        link_button("Support on Boosty", 11, Some(Msg::OpenUrl(BOOSTY_URL.into()))),
    ]
    .spacing(1)
    .width(Fill);

    let boosty_image = mouse_area(column![
        Space::new().height(8),
        container(BOOSTY.image().width(122)).center_x(Fill),
    ])
    .interaction(iced::mouse::Interaction::Pointer)
    .on_press(Msg::OpenUrl(BOOSTY_URL.into()));

    let panel = sunken_frame(
        container(row![info, Space::new().width(8), boosty_image].padding(8))
            .style(theme::panel)
            .width(Fill)
            .padding(4),
    );

    let thanks = text(
        "Thank you for your donations. All contributions go directly toward funding the station.",
    )
    .size(11)
    .font(BOLD)
    .center()
    .width(Fill);

    column![
        Space::new().height(8),
        title,
        Space::new().height(16),
        panel,
        Space::new().height(16),
        thanks,
        Space::new().height(16),
        container(close_button(wid)).center_x(Fill).padding([4, 2]),
    ]
    .padding(8)
    .width(Fill)
    .into()
}
