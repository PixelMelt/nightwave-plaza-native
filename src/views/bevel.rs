use crate::theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::event::Event;
use iced::{touch, Background, Border, Color, Element, Length, Padding, Rectangle, Shadow, Size};

pub struct Bevel<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Message>,
    menu: bool,
    width: Length,
    height: Length,
    padding: Padding,
    active: bool,
    drawn: Option<Look>,
}

#[derive(Default, Clone, Copy)]
struct State {
    is_pressed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Look {
    Flat,
    Raised,
    Pressed,
    MenuHover,
}

impl<'a, Message> Bevel<'a, Message> {
    fn new(content: impl Into<Element<'a, Message>>, menu: bool) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            menu,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding {
                top: 2.0,
                right: 6.0,
                bottom: 3.0,
                left: 6.0,
            },
            active: false,
            drawn: None,
        }
    }

    fn look(&self, state: &State, is_mouse_over: bool) -> Look {
        let enabled = self.on_press.is_some();
        if self.menu {
            if enabled && is_mouse_over {
                Look::MenuHover
            } else {
                Look::Flat
            }
        } else if self.active || (enabled && state.is_pressed && is_mouse_over) {
            Look::Pressed
        } else {
            Look::Raised
        }
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

impl<'a, Message: Clone + 'a> Widget<Message, iced::Theme, iced::Renderer> for Bevel<'a, Message> {
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
        renderer: &iced::Renderer,
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
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let look = self.look(state, cursor.is_over(bounds));

        if look != Look::Flat {
            quad(renderer, bounds, theme::BG_GRAY);
        }
        match look {
            Look::Raised => draw_symmetric_bevel(renderer, bounds, theme::BEVEL_RAISED),
            Look::Pressed => draw_symmetric_bevel(renderer, bounds, theme::BEVEL_PRESSED),
            Look::MenuHover => draw_thin_bevel(renderer, bounds, theme::THIN_MENU_HOVER),
            Look::Flat => {}
        }

        self.content.as_widget().draw(
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
        renderer: &iced::Renderer,
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

        let state = tree.state.downcast_mut::<State>();
        if !shell.is_event_captured() {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(msg) = self.on_press.clone() {
                        if cursor.is_over(layout.bounds()) {
                            state.is_pressed = true;
                            shell.publish(msg);
                            shell.capture_event();
                        }
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. }) => {
                    if state.is_pressed {
                        state.is_pressed = false;
                        shell.capture_event();
                    }
                }
                Event::Touch(touch::Event::FingerLost { .. })
                | Event::Mouse(mouse::Event::CursorLeft) => {
                    state.is_pressed = false;
                }
                _ => {}
            }
        }

        let look = self.look(state, cursor.is_over(layout.bounds()));
        if let Event::Window(iced::window::Event::RedrawRequested(_)) = event {
            self.drawn = Some(look);
        } else if self.drawn.is_some_and(|drawn| drawn != look) {
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

impl<'a, Message: Clone + 'a> From<Bevel<'a, Message>> for Element<'a, Message> {
    fn from(b: Bevel<'a, Message>) -> Self {
        Element::new(b)
    }
}

pub fn bevel_button<'a, Message>(content: impl Into<Element<'a, Message>>) -> Bevel<'a, Message> {
    Bevel::new(content, false)
}

pub fn menu_item<'a, Message>(content: impl Into<Element<'a, Message>>) -> Bevel<'a, Message> {
    Bevel::new(content, true).padding([5, 6])
}

pub fn draw_thin_bevel(renderer: &mut iced::Renderer, b: Rectangle, (tl, br): (Color, Color)) {
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

pub fn draw_symmetric_bevel(
    renderer: &mut iced::Renderer,
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

pub fn quad(renderer: &mut iced::Renderer, bounds: Rectangle, color: Color) {
    use iced::advanced::Renderer;
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
