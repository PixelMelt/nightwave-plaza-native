use crate::state::Msg;
use crate::theme;
use crate::views::widgets::{d3_raised, d3_sunken};
use iced::widget::{button, column, container, horizontal_space, image, row, text, Space};
use iced::{Element, Fill};

const BOOSTY_IMG: &[u8] = include_bytes!("../../assets/icons/boosty.png");
const BOOSTY_URL: &str = "https://boosty.to/nightwaveplaza";

pub fn view(wid: iced::window::Id) -> Element<'static, Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };
    let link_color = iced::Color::from_rgb(0.024, 0.271, 0.678);

    let title = text("Love Nightwave Plaza?")
        .size(14)
        .font(bold)
        .center()
        .width(Fill);

    let info_text = column![
        text("Support the radio station and future updates by donating via Boosty to receive special Discord rewards!")
            .size(11)
            .center()
            .width(Fill),
        Space::with_height(4),

        button(
            text("Support on Boosty")
                .size(11)
                .color(link_color)
                .center()
                .width(Fill)
        )
        .on_press(Msg::OpenUrl(BOOSTY_URL.into()))
        .style(move |_: &iced::Theme, _| iced::widget::button::Style {
            background: None,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            text_color: link_color,
        })
        .padding(0)
        .width(Fill),
    ]
    .spacing(1)
    .width(Fill);

    let boosty_image = button(column![
        Space::with_height(8),
        container(image(image::Handle::from_bytes(BOOSTY_IMG)).width(122)).center_x(Fill)
    ])
    .on_press(Msg::OpenUrl(BOOSTY_URL.into()))
    .style(|_: &iced::Theme, _| iced::widget::button::Style {
        background: None,
        border: iced::Border::default(),
        shadow: iced::Shadow::default(),
        text_color: theme::BLACK,
    })
    .padding(0);

    let panel_content = row![info_text, Space::with_width(8), boosty_image].padding(8);

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

    let close_btn = d3_raised(
        button(text("Close").size(11).center().width(80))
            .on_press(Msg::CloseWin(wid))
            .width(80)
            .style(theme::raised),
    );

    let bottom = row![horizontal_space(), close_btn, horizontal_space()].padding([4, 2]);

    column![
        Space::with_height(8),
        title,
        Space::with_height(16),
        panel,
        Space::with_height(16),
        thanks,
        Space::with_height(16),
        bottom,
    ]
    .padding(8)
    .width(Fill)
    .into()
}
