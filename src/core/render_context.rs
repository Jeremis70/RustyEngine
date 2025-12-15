use crate::core::color::Color;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;

/// CPU-side draw list. Collects vertices and clear color; no renderer coupling.
pub struct RenderContext {
    pub vertices: Vec<Vertex>,
    pub clear_color: Option<Color>,
    pub size: (u32, u32),
}

impl RenderContext {
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            vertices: Vec::new(),
            clear_color: None,
            size,
        }
    }

    /// Request screen clear at frame start.
    pub fn clear(&mut self, color: Color) {
        self.clear_color = Some(color);
    }

    /// Push a single vertex.
    pub fn push(&mut self, v: Vertex) {
        self.vertices.push(v);
    }

    /// Push many vertices (typical path for shapes).
    pub fn extend(&mut self, verts: &[Vertex]) {
        self.vertices.extend_from_slice(verts);
    }

    /// Convert pixel-space to NDC.
    pub fn to_ndc(&self, p: Vec2) -> Vec2 {
        let w = self.size.0.max(1) as f32;
        let h = self.size.1.max(1) as f32;

        Vec2 {
            x: (p.x / w) * 2.0 - 1.0,
            y: 1.0 - (p.y / h) * 2.0,
        }
    }
}
