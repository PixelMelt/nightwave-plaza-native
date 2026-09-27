use super::widgets::{Png, button};
use crate::message::Msg;
use crate::state::Plaza;
use crate::window::MessageIcon;
use iced::widget::{Space, column, container, row, text};
use iced::window::Id;
use iced::{Alignment, Element, Fill};

static ERROR: Png = Png::new(include_bytes!("../assets/icons/msg_error.png"));
static INFORMATION: Png = Png::new(include_bytes!("../assets/icons/msg_information_large.png"));

const ICON_SIZE: f32 = 32.0;

pub fn view(state: &Plaza, icon: MessageIcon, wid: Id) -> Element<'_, Msg> {
    let message = state.messages.get(&icon).map_or("", String::as_str);
    let image = match icon {
        MessageIcon::Error => &ERROR,
        MessageIcon::Information => &INFORMATION,
    };
    let body = row![
        image.image().width(ICON_SIZE).height(ICON_SIZE),
        Space::new().width(12),
        container(text(message).size(11))
            .align_y(Alignment::Center)
            .height(Fill)
            .width(Fill),
    ]
    .height(Fill);
    column![
        body,
        Space::new().height(12),
        container(button("OK", 76).on_press(Msg::CloseWindow(wid))).center_x(Fill),
    ]
    .padding(12)
    .into()
}
