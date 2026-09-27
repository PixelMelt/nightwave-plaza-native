use super::paint;
use crate::theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse};
use iced::event::Event;
use iced::{Element, Length, Rectangle, Size, touch};

const HEIGHT: f32 = 26.0;
const GROOVE_Y: f32 = 10.0;
const GROOVE_HEIGHT: f32 = 4.0;
const HANDLE_WIDTH: f32 = 12.0;
const HANDLE_HEIGHT: f32 = 24.0;
const HANDLE_Y: f32 = 1.0;
const ICON_WIDTH: f32 = 11.0;
const ICON_HEIGHT: f32 = 16.0;
const ICON_Y: f32 = 5.0;
const ICON_SPACE: f32 = 18.0;

#[derive(Default, Clone, Copy)]
struct State {
    dragging: bool,
}

pub struct VolumeSlider<'a, Message> {
    value: f32,
    icon: Element<'a, Message>,
    on_change: Box<dyn Fn(f32) -> Message + 'a>,
    drawn_over_handle: Option<bool>,
}

pub fn volume_slider<'a, Message>(
    value: f32,
    icon: impl Into<Element<'a, Message>>,
    on_change: impl Fn(f32) -> Message + 'a,
) -> VolumeSlider<'a, Message> {
    VolumeSlider {
        value,
        icon: icon.into(),
        on_change: Box::new(on_change),
        drawn_over_handle: None,
    }
}

fn line_width(width: f32) -> f32 {
    (width - ICON_SPACE).max(HANDLE_WIDTH + 1.0)
}

fn percent_at(bounds: Rectangle, x: f32) -> f32 {
    let travel = (line_width(bounds.width) - HANDLE_WIDTH).max(1.0);
    ((x - bounds.x) / travel * 100.0).clamp(0.0, 100.0).round()
}

impl<Message> VolumeSlider<'_, Message> {
    fn handle_bounds(&self, bounds: Rectangle) -> Rectangle {
        let travel = (line_width(bounds.width) - HANDLE_WIDTH).max(0.0);
        Rectangle {
            x: bounds.x + travel * (self.value / 100.0).clamp(0.0, 1.0),
            y: bounds.y + HANDLE_Y,
            width: HANDLE_WIDTH,
            height: HANDLE_HEIGHT,
        }
    }

    fn over_handle(&self, cursor: mouse::Cursor, bounds: Rectangle) -> bool {
        cursor
            .position()
            .is_some_and(|p| self.handle_bounds(bounds).contains(p))
    }

    #[expect(
        clippy::float_cmp,
        reason = "both sides are whole percentages, which f32 represents exactly"
    )]
    fn publish_at(&self, bounds: Rectangle, x: f32, shell: &mut Shell<'_, Message>) {
        let percent = percent_at(bounds, x);
        if percent != self.value {
            shell.publish((self.on_change)(percent));
        }
    }
}

impl<'a, Message: 'a> Widget<Message, iced::Theme, iced::Renderer> for VolumeSlider<'a, Message> {
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
            width: Length::Fill,
            height: Length::Fixed(HEIGHT),
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits
            .width(Length::Fill)
            .height(Length::Fixed(HEIGHT))
            .resolve(Length::Fill, HEIGHT, Size::ZERO);

        let icon_limits = layout::Limits::new(Size::ZERO, Size::new(ICON_WIDTH, ICON_HEIGHT))
            .width(Length::Fixed(ICON_WIDTH))
            .height(Length::Fixed(ICON_HEIGHT));
        let icon_node = self
            .icon
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &icon_limits)
            .move_to((size.width - ICON_WIDTH, ICON_Y));

        layout::Node::with_children(size, vec![icon_node])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        let line_width = line_width(b.width);
        let rect = |x, y, width, height| Rectangle {
            x,
            y,
            width,
            height,
        };

        let groove = rect(b.x, b.y + GROOVE_Y, line_width, GROOVE_HEIGHT);
        paint::thin_bevel(renderer, groove, theme::THIN_SUNKEN);
        let groove_inside = rect(
            groove.x + 1.0,
            groove.y + 1.0,
            groove.width - 2.0,
            groove.height - 2.0,
        );
        paint::fill_rect(renderer, groove_inside, theme::BG_GRAY);

        let handle = self.handle_bounds(b);
        paint::fill_rect(renderer, handle, theme::BG_GRAY);
        paint::bevel(renderer, handle, theme::BEVEL_RAISED);

        self.icon.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().expect("widget has one child"),
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.icon.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().expect("widget has one child"),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        if !shell.is_event_captured() {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(p) = cursor.position_over(bounds) {
                        state.dragging = true;
                        self.publish_at(bounds, p.x, shell);
                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    if state.dragging
                        && let Some(p) = cursor.position()
                    {
                        self.publish_at(bounds, p.x, shell);
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(
                    touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. },
                ) if state.dragging => {
                    state.dragging = false;
                    shell.capture_event();
                }
                _ => {}
            }
        }

        let over_handle = self.over_handle(cursor, bounds);
        if let Event::Window(iced::window::Event::RedrawRequested(_)) = event {
            self.drawn_over_handle = Some(over_handle);
        } else if self
            .drawn_over_handle
            .is_some_and(|drawn| drawn != over_handle)
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.over_handle(cursor, layout.bounds()) {
            return mouse::Interaction::Pointer;
        }
        self.icon.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().expect("widget has one child"),
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message: 'a> From<VolumeSlider<'a, Message>> for Element<'a, Message> {
    fn from(s: VolumeSlider<'a, Message>) -> Self {
        Element::new(s)
    }
}
