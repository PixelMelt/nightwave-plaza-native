use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::event::Event;
use iced::{
    touch, Background, Border, Color, Element, Length, Padding, Rectangle, Shadow, Size,
};

use crate::theme;

#[derive(Clone, Copy)]
enum BevelStyle {
    Object,
    TitleButton,
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
    status: Option<(BevelKind, bool)>,
}

impl<'a, Message, Theme, Renderer> Bevel<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            style: BevelStyle::Object,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding {
                top: 2.0,
                right: 6.0,
                bottom: 3.0,
                left: 6.0,
            },
            active: false,
            status: None,
        }
    }

    fn kind(&self, is_pressed: bool, is_mouse_over: bool) -> BevelKind {
        match self.style {
            BevelStyle::Object | BevelStyle::TitleButton => {
                if is_pressed {
                    BevelKind::Pressed
                } else {
                    BevelKind::Object
                }
            }
            BevelStyle::Menu => {
                if self.on_press.is_some() && is_mouse_over {
                    BevelKind::MenuHover
                } else {
                    BevelKind::None
                }
            }
        }
    }

    fn visual(&self, state: &State, is_mouse_over: bool) -> (BevelKind, bool) {
        let is_pressed =
            (self.on_press.is_some() && state.is_pressed && is_mouse_over) || self.active;
        (self.kind(is_pressed, is_mouse_over), is_pressed)
    }

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

    fn style(mut self, style: BevelStyle) -> Self {
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
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
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
        let (bevel, _) = self.visual(state, cursor.is_over(bounds));

        if !matches!(bevel, BevelKind::None) {
            quad(renderer, bounds, theme::BG_GRAY);
        }

        match bevel {
            BevelKind::Object => draw_symmetric_bevel(renderer, bounds, theme::BEVEL_RAISED),
            BevelKind::MenuHover => draw_thin_bevel(renderer, bounds, theme::THIN_MENU_HOVER),
            BevelKind::Pressed => draw_symmetric_bevel(renderer, bounds, theme::BEVEL_PRESSED),
            BevelKind::None => {}
        }

        let content_layout = layout.children().next().unwrap();
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            content_layout,
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
        self.content.as_widget_mut().update(
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
                    if self.on_press.is_some() && cursor.is_over(layout.bounds()) {
                        let state = tree.state.downcast_mut::<State>();
                        state.is_pressed = true;
                        if let Some(msg) = self.on_press.clone() {
                            shell.publish(msg);
                        }
                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. }) => {
                    let state = tree.state.downcast_mut::<State>();
                    if state.is_pressed {
                        state.is_pressed = false;
                        shell.capture_event();
                    }
                }
                Event::Touch(touch::Event::FingerLost { .. })
                | Event::Mouse(mouse::Event::CursorLeft) => {
                    tree.state.downcast_mut::<State>().is_pressed = false;
                }
                _ => {}
            }
        }

        let state = tree.state.downcast_ref::<State>();
        let current = self.visual(state, cursor.is_over(layout.bounds()));
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum BevelKind {
    None,
    Object,
    MenuHover,
    Pressed,
}

pub(crate) fn draw_thin_bevel<R: iced::advanced::Renderer>(
    renderer: &mut R,
    b: Rectangle,
    (tl, br): (Color, Color),
) {
    let edge = |x, y, width, height| Rectangle {
        x,
        y,
        width,
        height,
    };
    quad(renderer, edge(b.x, b.y, b.width, 1.0), tl);
    quad(renderer, edge(b.x, b.y, 1.0, b.height), tl);
    quad(renderer, edge(b.x, b.y + b.height - 1.0, b.width, 1.0), br);
    quad(renderer, edge(b.x + b.width - 1.0, b.y, 1.0, b.height), br);
}

pub(crate) fn draw_symmetric_bevel<R: iced::advanced::Renderer>(
    renderer: &mut R,
    bounds: Rectangle,
    c: theme::BevelColors,
) {
    draw_thin_bevel(renderer, bounds, (c.tl_outer, c.br_outer));
    let inner = Rectangle {
        x: bounds.x + 1.0,
        y: bounds.y + 1.0,
        width: bounds.width - 2.0,
        height: bounds.height - 2.0,
    };
    draw_thin_bevel(renderer, inner, (c.tl_inner, c.br_inner));
}

pub(crate) fn quad<R: iced::advanced::Renderer>(renderer: &mut R, bounds: Rectangle, color: Color) {
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
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
