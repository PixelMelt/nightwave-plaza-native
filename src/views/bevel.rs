//! Custom interactive bevel widget — a button replacement that owns its 3D
//! bevel rendering so the pressed state can flip the bevel entirely (matching
//! plaza's Win9x button behaviour), and a menu-item style that draws the
//! d3-window raised bevel only on hover. Bevel strips are drawn directly via
//! `renderer.fill_quad` rather than nested containers.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::event::{self, Event};
use iced::{
    touch, Background, Border, Color, Element, Length, Padding, Rectangle, Shadow, Size, Vector,
};

use crate::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BevelStyle {
    /// d3-object — raised button bevel that inverts to all-black + inset
    /// dark-gray ring while held down.
    Object,
    /// Thin (1px) raised bevel — used for tight title-bar buttons where the
    /// full 2-pixel d3-object ring looks chunky and pushes the icon down.
    /// Pressed state swaps to a 1px black-on-DARK_GRAY look.
    TitleButton,
    /// menu item — no bevel normally, d3-window raised bevel on hover.
    Menu,
}

#[derive(Default, Clone, Copy)]
struct State {
    is_pressed: bool,
}

pub struct Bevel<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<Message>,
    style: BevelStyle,
    width: Length,
    height: Length,
    padding: Padding,
    active: bool,
}

impl<'a, Message, Theme, Renderer> Bevel<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            style: BevelStyle::Object,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(4.0),
            active: false,
        }
    }

    /// Force the pressed/active visual regardless of pointer state. Useful
    /// for "selected tab" buttons where the bevel should permanently look
    /// pressed.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    pub fn maybe_on_press(mut self, msg: Option<Message>) -> Self {
        self.on_press = msg;
        self
    }

    pub fn style(mut self, style: BevelStyle) -> Self {
        self.style = style;
        self
    }

    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }

    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = p.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Bevel<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Plaza's CSS pads each button to the value it specifies (eg `.action`
        // = 5px 6px, `.win-button` = 4px). The 2px bevel is drawn *inside*
        // that padding via `border + inset box-shadow`, so it does not
        // increase the element's size. Match that here: use `self.padding`
        // directly and let the bevel ride on top of the outer pixels of the
        // content's padding area.
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        })
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
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let is_mouse_over = cursor.is_over(bounds);
        let interactive = self.on_press.is_some();
        let is_pressed =
            (interactive && state.is_pressed && is_mouse_over) || self.active;

        let bevel = match self.style {
            BevelStyle::Object => {
                if is_pressed {
                    BevelKind::Pressed
                } else {
                    BevelKind::Object
                }
            }
            BevelStyle::TitleButton => {
                if is_pressed {
                    BevelKind::ThinPressed
                } else {
                    BevelKind::Thin
                }
            }
            BevelStyle::Menu => {
                if interactive && is_mouse_over {
                    BevelKind::Window
                } else {
                    BevelKind::None
                }
            }
        };

        // Fill face background for any state that has a visible bevel. The
        // button uses BG_GRAY so its bevel doesn't show through to whatever
        // is behind the row.
        if !matches!(bevel, BevelKind::None) {
            quad(renderer, bounds, theme::BG_GRAY);
        }

        match bevel {
            BevelKind::Object => draw_symmetric_bevel(
                renderer,
                bounds,
                theme::WHITE,
                theme::LIGHT_GRAY,
                theme::DARK_GRAY,
                theme::BLACK,
            ),
            BevelKind::Window => draw_symmetric_bevel(
                renderer,
                bounds,
                theme::LIGHT_GRAY,
                theme::WHITE,
                theme::DARK_GRAY,
                theme::BLACK,
            ),
            BevelKind::Thin => draw_thin_bevel(
                renderer,
                bounds,
                theme::WHITE,
                theme::BLACK,
            ),
            BevelKind::ThinPressed => draw_thin_bevel(
                renderer,
                bounds,
                theme::BLACK,
                theme::DARK_GRAY,
            ),
            BevelKind::Pressed => draw_pressed_bevel(renderer, bounds),
            BevelKind::None => {}
        }

        // When pressed, shift the inner content down+right by 1px to give the
        // "pushed in" feel that real Win9x buttons have.
        let content_layout = layout.children().next().unwrap();
        let translation = if is_pressed {
            Vector::new(1.0, 1.0)
        } else {
            Vector::new(0.0, 0.0)
        };
        renderer.with_translation(translation, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                content_layout,
                cursor,
                viewport,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() && cursor.is_over(layout.bounds()) {
                    tree.state.downcast_mut::<State>().is_pressed = true;
                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                let state = tree.state.downcast_mut::<State>();
                if state.is_pressed {
                    state.is_pressed = false;
                    if let Some(msg) = self.on_press.clone() {
                        if cursor.is_over(layout.bounds()) {
                            shell.publish(msg);
                        }
                    }
                    return event::Status::Captured;
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                tree.state.downcast_mut::<State>().is_pressed = false;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.on_press.is_some() && cursor.is_over(layout.bounds()) {
            return mouse::Interaction::Pointer;
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
enum BevelKind {
    None,
    Object,
    Window,
    Thin,
    ThinPressed,
    Pressed,
}

fn draw_thin_bevel<R: iced::advanced::Renderer>(
    renderer: &mut R,
    bounds: Rectangle,
    tl: Color,
    br: Color,
) {
    let Rectangle { x, y, width: w, height: h } = bounds;
    quad(renderer, Rectangle { x, y, width: w, height: 1.0 }, tl);
    quad(renderer, Rectangle { x, y, width: 1.0, height: h }, tl);
    quad(
        renderer,
        Rectangle { x, y: y + h - 1.0, width: w, height: 1.0 },
        br,
    );
    quad(
        renderer,
        Rectangle { x: x + w - 1.0, y, width: 1.0, height: h },
        br,
    );
}

fn draw_symmetric_bevel<R: iced::advanced::Renderer>(
    renderer: &mut R,
    bounds: Rectangle,
    tl_outer: Color,
    tl_inner: Color,
    br_inner: Color,
    br_outer: Color,
) {
    let Rectangle { x, y, width: w, height: h } = bounds;

    // Outer ring (1px) — top, left, bottom, right
    quad(renderer, Rectangle { x, y, width: w, height: 1.0 }, tl_outer);
    quad(renderer, Rectangle { x, y, width: 1.0, height: h }, tl_outer);
    quad(
        renderer,
        Rectangle { x, y: y + h - 1.0, width: w, height: 1.0 },
        br_outer,
    );
    quad(
        renderer,
        Rectangle { x: x + w - 1.0, y, width: 1.0, height: h },
        br_outer,
    );

    // Inner ring (1px, inside the outer)
    quad(
        renderer,
        Rectangle { x: x + 1.0, y: y + 1.0, width: w - 2.0, height: 1.0 },
        tl_inner,
    );
    quad(
        renderer,
        Rectangle { x: x + 1.0, y: y + 1.0, width: 1.0, height: h - 2.0 },
        tl_inner,
    );
    quad(
        renderer,
        Rectangle { x: x + 1.0, y: y + h - 2.0, width: w - 2.0, height: 1.0 },
        br_inner,
    );
    quad(
        renderer,
        Rectangle { x: x + w - 2.0, y: y + 1.0, width: 1.0, height: h - 2.0 },
        br_inner,
    );
}

fn draw_pressed_bevel<R: iced::advanced::Renderer>(renderer: &mut R, bounds: Rectangle) {
    let Rectangle { x, y, width: w, height: h } = bounds;

    // All-black outer ring.
    quad(renderer, Rectangle { x, y, width: w, height: 1.0 }, theme::BLACK);
    quad(
        renderer,
        Rectangle { x, y: y + h - 1.0, width: w, height: 1.0 },
        theme::BLACK,
    );
    quad(renderer, Rectangle { x, y, width: 1.0, height: h }, theme::BLACK);
    quad(
        renderer,
        Rectangle { x: x + w - 1.0, y, width: 1.0, height: h },
        theme::BLACK,
    );

    // Inset 1px dark-gray ring just inside the black.
    quad(
        renderer,
        Rectangle { x: x + 1.0, y: y + 1.0, width: w - 2.0, height: 1.0 },
        theme::DARK_GRAY,
    );
    quad(
        renderer,
        Rectangle { x: x + 1.0, y: y + h - 2.0, width: w - 2.0, height: 1.0 },
        theme::DARK_GRAY,
    );
    quad(
        renderer,
        Rectangle { x: x + 1.0, y: y + 1.0, width: 1.0, height: h - 2.0 },
        theme::DARK_GRAY,
    );
    quad(
        renderer,
        Rectangle { x: x + w - 2.0, y: y + 1.0, width: 1.0, height: h - 2.0 },
        theme::DARK_GRAY,
    );
}

fn quad<R: iced::advanced::Renderer>(renderer: &mut R, bounds: Rectangle, color: Color) {
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        Background::Color(color),
    );
}

impl<'a, Message, Theme, Renderer> From<Bevel<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(b: Bevel<'a, Message, Theme, Renderer>) -> Self {
        Element::new(b)
    }
}

pub fn bevel_button<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Bevel<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Bevel::new(content).style(BevelStyle::Object)
}

pub fn menu_item<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Bevel<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Bevel::new(content)
        .style(BevelStyle::Menu)
        .padding(Padding::from([5, 6]))
}

pub fn title_button<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Bevel<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Bevel::new(content)
        .style(BevelStyle::TitleButton)
        .padding(Padding::from(1))
}
