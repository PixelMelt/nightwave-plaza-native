use crate::state::Msg;
use crate::theme;
use crate::views::{bold_font, close_btn, d3_sunken, link_button, static_image};
use iced::widget::{column, container, image, mouse_area, row, text, Space};
use iced::{Element, Fill};

const BOOSTY_IMG: &[u8] = include_bytes!("../assets/img/boosty.png");
const BOOSTY_URL: &str = "https://boosty.to/nightwaveplaza";

pub fn view(wid: iced::window::Id) -> Element<'static, Msg> {
    let bold = bold_font();

    let title = text("Love Nightwave Plaza?")
        .size(14)
        .font(bold)
        .center()
        .width(Fill);

    let info_text = column![
        text("Support the radio station and future updates by donating via Boosty to receive special Discord rewards!")
            .size(11).center().width(Fill),
        Space::new().height(4),
        link_button("Support on Boosty", 11, Some(Msg::OpenUrl(BOOSTY_URL.into()))),
    ]
    .spacing(1)
    .width(Fill);

    let boosty_image = mouse_area(column![
        Space::new().height(8),
        container(image(static_image(BOOSTY_IMG)).width(122)).center_x(Fill),
    ])
    .interaction(iced::mouse::Interaction::Pointer)
    .on_press(Msg::OpenUrl(BOOSTY_URL.into()));

    let panel_content = row![info_text, Space::new().width(8), boosty_image].padding(8);

    let panel = d3_sunken(
        container(panel_content)
            .style(theme::panel)
            .width(Fill)
            .padding(4),
    );

    let thanks = text(
        "Thank you for your donations. All contributions go directly toward funding the station.",
    )
    .size(11)
    .font(bold)
    .center()
    .width(Fill);

    let bottom = row![
        Space::new().width(iced::Fill),
        close_btn(wid),
        Space::new().width(iced::Fill)
    ]
    .padding([4, 2]);

    column![
        Space::new().height(8),
        title,
        Space::new().height(16),
        panel,
        Space::new().height(16),
        thanks,
        Space::new().height(16),
        bottom,
    ]
    .padding(8)
    .width(Fill)
    .into()
}
