use crate::{
    core::assets::font::FontId,
    math::{Color, Transform},
    render::{Drawable, RenderContext, Transform2d},
};

pub struct Text {
    pub font: FontId,
    pub content: String,
    pub font_size: u32,
    pub color: Color,
    pub transform: Transform,
}

impl Text {
    pub fn new(font: FontId, content: &str, font_size: u32, color: Color) -> Self {
        Self {
            font,
            content: content.to_string(),
            font_size,
            color,
            transform: Transform::new(),
        }
    }
}

impl Transform2d for Text {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Drawable for Text {
    fn draw(&self, ctx: &mut RenderContext) {
        todo!()
    }
}
