#[derive(Debug, Clone)]
pub struct Map {
    pub tile_size: f32,
    pub grid: Vec<Vec<u8>>,
}

impl Map {
    pub fn demo(tile_size: f32) -> Self {
        let grid: Vec<Vec<u8>> = vec![
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 3, 3, 3, 3, 0, 0, 0, 2, 2, 2, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 2, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 2, 0, 0, 1],
            vec![1, 0, 0, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 1, 1, 3, 1, 3, 1, 1, 1, 3, 0, 0, 3, 1, 1, 1],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 0, 0, 3, 1, 1, 1],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 0, 0, 3, 1, 1, 1],
            vec![1, 1, 3, 1, 1, 1, 1, 1, 1, 3, 0, 0, 3, 1, 1, 1],
            vec![1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 2, 0, 0, 0, 0, 0, 3, 4, 0, 4, 3, 0, 1],
            vec![1, 0, 0, 5, 0, 0, 0, 0, 0, 0, 3, 0, 3, 0, 0, 1],
            vec![1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 4, 0, 0, 0, 1],
            vec![1, 1, 3, 3, 0, 0, 3, 3, 1, 3, 3, 1, 3, 1, 1, 1],
            vec![1, 1, 1, 3, 0, 0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![1, 3, 3, 4, 0, 0, 4, 3, 3, 3, 3, 3, 3, 3, 3, 1],
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
            vec![3, 0, 0, 5, 0, 0, 0, 5, 0, 0, 0, 5, 0, 0, 0, 3],
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
            vec![3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
        ];

        Self { tile_size, grid }
    }

    pub fn width(&self) -> usize {
        self.grid.first().map(|r| r.len()).unwrap_or(0)
    }

    pub fn height(&self) -> usize {
        self.grid.len()
    }

    pub fn is_wall(&self, x: usize, y: usize) -> bool {
        self.grid
            .get(y)
            .and_then(|row| row.get(x))
            .is_some_and(|v| *v != 0)
    }

    pub fn is_wall_i32(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return true;
        }
        let (xu, yu) = (x as usize, y as usize);
        if yu >= self.height() || xu >= self.width() {
            return true;
        }
        self.is_wall(xu, yu)
    }

    pub fn wall_cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.grid.iter().enumerate().flat_map(|(y, row)| {
            row.iter().enumerate().filter_map(
                move |(x, cell)| {
                    if *cell != 0 { Some((x, y)) } else { None }
                },
            )
        })
    }

    pub fn draw(&self, ctx: &mut crate::render::context::RenderContext) {
        use crate::render::Drawable;
        use crate::render::shapes::Rectangle;

        let s = super::settings::settings();

        for (x, y) in self.wall_cells() {
            Rectangle::new_outline(
                crate::math::vec2::Vec2::new(x as f32 * self.tile_size, y as f32 * self.tile_size),
                crate::math::vec2::Vec2::new(self.tile_size, self.tile_size),
                s.wall_outline_color,
                s.wall_outline_thickness,
            )
            .draw(ctx);
        }
    }
}
