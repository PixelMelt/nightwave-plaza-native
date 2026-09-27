use crate::theme::BevelColors;
use iced::advanced::Renderer as _;
use iced::advanced::renderer::Quad;
use iced::{Background, Border, Color, Rectangle, Renderer, Shadow};

pub fn fill_rect(renderer: &mut Renderer, bounds: Rectangle, color: Color) {
    renderer.fill_quad(
        Quad {
            bounds,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
        Background::Color(color),
    );
}

pub fn thin_bevel(renderer: &mut Renderer, b: Rectangle, (top_left, bottom_right): (Color, Color)) {
    let rect = |x, y, width, height| Rectangle {
        x,
        y,
        width,
        height,
    };
    fill_rect(renderer, rect(b.x, b.y, b.width, 1.0), top_left);
    fill_rect(renderer, rect(b.x, b.y, 1.0, b.height), top_left);
    fill_rect(
        renderer,
        rect(b.x, b.y + b.height - 1.0, b.width, 1.0),
        bottom_right,
    );
    fill_rect(
        renderer,
        rect(b.x + b.width - 1.0, b.y, 1.0, b.height),
        bottom_right,
    );
}

pub fn bevel(renderer: &mut Renderer, bounds: Rectangle, colors: BevelColors) {
    thin_bevel(renderer, bounds, (colors.tl_outer, colors.br_outer));
    let inner = Rectangle {
        x: bounds.x + 1.0,
        y: bounds.y + 1.0,
        width: bounds.width - 2.0,
        height: bounds.height - 2.0,
    };
    thin_bevel(renderer, inner, (colors.tl_inner, colors.br_inner));
}
