use crate::state::{ExportMsg, Msg, Plaza};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{close_btn_padded, d3_sunken, LINK_COLOR};
use iced::widget::{button, column, container, text, Space};
use iced::{Element, Fill, Theme};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let body: Element<Msg> = if let Some(ref link) = state.export.link {
        let link = link.clone();
        column![
            text("Export successful! Your file is ready to download.")
                .size(11)
                .center()
                .width(Fill),
            Space::with_height(8),
            button(
                text("Download")
                    .size(11)
                    .color(LINK_COLOR)
                    .center()
                    .width(Fill)
            )
            .on_press(Msg::OpenUrl(link))
            .style(|_: &Theme, _| button::Style {
                background: None,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                text_color: LINK_COLOR,
            })
            .padding(0)
            .width(Fill),
        ]
        .width(Fill)
        .into()
    } else if state.export.loading {
        column![text("Exporting...").size(11).center().width(Fill)]
            .width(Fill)
            .into()
    } else {
        column![
            text("Export your favorites list as a CSV file. Click below to begin.")
                .size(11)
                .center()
                .width(Fill),
            Space::with_height(10),
            container(
                bevel_button(text("Export").size(11).center().width(90))
                    .on_press(Msg::Export(ExportMsg::Start))
                    .width(90)
            )
            .center_x(Fill),
        ]
        .width(Fill)
        .into()
    };

    let mut col = column![body].width(Fill);
    if let Some(ref err) = state.export.error {
        col = col.push(Space::with_height(6));
        col = col.push(
            text(err)
                .size(11)
                .color(iced::Color::from_rgb(0.8, 0.0, 0.0))
                .center()
                .width(Fill),
        );
    }

    let panel = d3_sunken(container(col).style(theme::panel).width(Fill).padding(12));

    column![
        panel,
        Space::with_height(12),
        container(close_btn_padded(wid)).center_x(Fill),
    ]
    .padding(8)
    .width(Fill)
    .into()
}
