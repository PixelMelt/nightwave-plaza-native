use crate::state::{ExportMsg, Msg, Plaza};
use crate::theme;
use crate::theme::ERROR_RED;
use crate::views::bevel_button;
use crate::views::{close_btn_padded, d3_sunken, link_button};
use iced::widget::{column, container, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let body: Element<Msg> = if let Some(ref link) = state.export.link {
        column![
            text("Export successful! Your file is ready to download.")
                .size(11)
                .center()
                .width(Fill),
            Space::new().height(8),
            link_button("Download", 11, Some(Msg::OpenUrl(link.clone()))),
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
            Space::new().height(10),
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
        col = col.push(Space::new().height(6));
        col = col.push(text(err).size(11).color(ERROR_RED).center().width(Fill));
    }

    let panel = d3_sunken(container(col).style(theme::panel).width(Fill).padding(12));

    column![
        panel,
        Space::new().height(12),
        container(close_btn_padded(wid)).center_x(Fill),
    ]
    .padding(8)
    .width(Fill)
    .into()
}
