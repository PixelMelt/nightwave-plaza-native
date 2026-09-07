use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::event::Event;
use iced::{touch, Element, Length, Rectangle, Size};

use crate::theme;
use crate::views::bevel::{draw_symmetric_bevel, quad};

// Matches the web noUiSlider markup in WinPlayerVolume.vue:
// 26px tall control, 4px sunken line at y=10, 12x24 raised handle at y=1,
// 11x16 volume icon at the right edge (y=5), 7px gap between line and icon.
const HEIGHT: f32 = 26.0;
const LINE_Y: f32 = 10.0;
const LINE_H: f32 = 4.0;
const HANDLE_W: f32 = 12.0;
const HANDLE_H: f32 = 24.0;
const HANDLE_Y: f32 = 1.0;
const ICON_W: f32 = 11.0;
const ICON_H: f32 = 16.0;
const ICON_Y: f32 = 5.0;
const RIGHT_PAD: f32 = 18.0;

#[derive(Default, Clone, Copy)]
struct State {
    dragging: bool,
}

pub struct VolumeSlider<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    value: f32,
    icon: Element<'a, Message, Theme, Renderer>,
    on_change: Box<dyn Fn(f32) -> Message + 'a>,
    width: Length,
    status: Option<(bool, bool)>,
}

impl<'a, Message, Theme, Renderer> VolumeSlider<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn line_width(&self, width: f32) -> f32 {
        (width - RIGHT_PAD).max(HANDLE_W + 1.0)
    }

    fn handle_bounds(&self, bounds: Rectangle) -> Rectangle {
        let travel = (self.line_width(bounds.width) - HANDLE_W).max(0.0);
        Rectangle {
            x: bounds.x + travel * (self.value / 100.0).clamp(0.0, 1.0),
            y: bounds.y + HANDLE_Y,
            width: HANDLE_W,
            height: HANDLE_H,
        }
    }

    fn value_at(&self, bounds: Rectangle, x: f32) -> f32 {
        let travel = (self.line_width(bounds.width) - HANDLE_W).max(1.0);
        ((x - bounds.x) / travel * 100.0).clamp(0.0, 100.0).round()
    }

    fn visual(&self, state: &State, cursor: mouse::Cursor, bounds: Rectangle) -> (bool, bool) {
        let over_handle = cursor
            .position()
            .is_some_and(|p| self.handle_bounds(bounds).contains(p));
        (over_handle, state.dragging)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for VolumeSlider<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.icon)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.icon));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Fixed(HEIGHT),
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits
            .width(self.width)
            .height(Length::Fixed(HEIGHT))
            .resolve(self.width, HEIGHT, Size::ZERO);

        let icon_limits = layout::Limits::new(Size::ZERO, Size::new(ICON_W, ICON_H))
            .width(Length::Fixed(ICON_W))
            .height(Length::Fixed(ICON_H));
        let icon_node = self
            .icon
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &icon_limits)
            .move_to((size.width - ICON_W, ICON_Y));

        layout::Node::with_children(size, vec![icon_node])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        let line_w = self.line_width(b.width);

        // sunken line
        quad(renderer, Rectangle { x: b.x, y: b.y + LINE_Y, width: line_w, height: 1.0 }, theme::DARK_GRAY);
        quad(renderer, Rectangle { x: b.x, y: b.y + LINE_Y, width: 1.0, height: LINE_H }, theme::DARK_GRAY);
        quad(renderer, Rectangle { x: b.x, y: b.y + LINE_Y + LINE_H - 1.0, width: line_w, height: 1.0 }, theme::WHITE);
        quad(renderer, Rectangle { x: b.x + line_w - 1.0, y: b.y + LINE_Y, width: 1.0, height: LINE_H }, theme::WHITE);
        quad(renderer, Rectangle { x: b.x + 1.0, y: b.y + LINE_Y + 1.0, width: line_w - 2.0, height: LINE_H - 2.0 }, theme::BG_GRAY);

        // raised handle
        let handle = self.handle_bounds(b);
        quad(renderer, handle, theme::BG_GRAY);
        draw_symmetric_bevel(renderer, handle, theme::BEVEL_RAISED);

        self.icon.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.icon.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if !shell.is_event_captured() {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(p) = cursor.position_over(layout.bounds()) {
                        let state = tree.state.downcast_mut::<State>();
                        state.dragging = true;
                        let v = self.value_at(layout.bounds(), p.x);
                        if v != self.value {
                            shell.publish((self.on_change)(v));
                        }
                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    let state = tree.state.downcast_ref::<State>();
                    if state.dragging {
                        if let Some(p) = cursor.position() {
                            let v = self.value_at(layout.bounds(), p.x);
                            if v != self.value {
                                shell.publish((self.on_change)(v));
                            }
                        }
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    let state = tree.state.downcast_mut::<State>();
                    if state.dragging {
                        state.dragging = false;
                        shell.capture_event();
                    }
                }
                _ => {}
            }
        }

        let state = tree.state.downcast_ref::<State>();
        let current = self.visual(state, cursor, layout.bounds());
        if let Event::Window(iced::window::Event::RedrawRequested(_)) = event {
            self.status = Some(current);
        } else if self.status.is_some_and(|status| status != current) {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor
            .position()
            .is_some_and(|p| self.handle_bounds(layout.bounds()).contains(p))
        {
            return mouse::Interaction::Pointer;
        }
        self.icon.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<VolumeSlider<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(s: VolumeSlider<'a, Message, Theme, Renderer>) -> Self {
        Element::new(s)
    }
}

pub fn volume_slider<'a, Message, Theme, Renderer>(
    value: f32,
    icon: impl Into<Element<'a, Message, Theme, Renderer>>,
    on_change: impl Fn(f32) -> Message + 'a,
) -> VolumeSlider<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    VolumeSlider {
        value,
        icon: icon.into(),
        on_change: Box::new(on_change),
        width: Length::Fill,
        status: None,
    }
}
