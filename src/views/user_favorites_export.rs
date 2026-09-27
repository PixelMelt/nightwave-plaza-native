use super::widgets::{button, link_button, sunken_panel};
use crate::message::{ExportMsg, Msg};
use crate::state::Plaza;
use crate::theme;
use iced::widget::{Space, column, container, text};
use iced::window::Id;
use iced::{Element, Fill, Length};

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let export = &state.export;
    let message = |s| text(s).size(11).center().width(Fill);

    let mut body = column![].width(Fill);
    if let Some(link) = &export.link {
        body = body
            .push(message(
                "Export successful! Your file is ready to download.",
            ))
            .push(Space::new().height(8))
            .push(link_button(
                "Download",
                11,
                Some(Msg::OpenUrl(link.clone())),
            ));
    } else if export.loading {
        body = body.push(message("Exporting..."));
    } else {
        body = body
            .push(message(
                "Export your favorites list as a CSV file. Click below to begin.",
            ))
            .push(Space::new().height(10))
            .push(
                container(button("Export", 90).on_press(Msg::Export(ExportMsg::Start)))
                    .center_x(Fill),
            );
    }
    if let Some(err) = &export.error {
        body = body
            .push(Space::new().height(6))
            .push(message(err).color(theme::ERROR_RED));
    }

    let close = button("Close", Length::Shrink)
        .on_press(Msg::CloseWindow(wid))
        .padding([4, 24]);

    column![
        sunken_panel(container(body).padding(4)),
        Space::new().height(12),
        container(close).center_x(Fill),
    ]
    .padding(8)
    .width(Fill)
    .into()
}
