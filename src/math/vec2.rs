// math/vec2.rs
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn scale(self, s: f32) -> Vec2 {
        Vec2 {
            x: self.x * s,
            y: self.y * s,
        }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(self) -> Vec2 {
        let len = self.length().max(1e-6);
        self.scale(1.0 / len)
    }

    pub fn to_array(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

// Helper trait to route Vec2 multiplication; public so Mul's output type can reference it.
pub trait Vec2Mul<Rhs> {
    type Output;
    fn vec2_mul(self, rhs: Rhs) -> Self::Output;
}

impl Vec2Mul<f32> for Vec2 {
    type Output = Vec2;

    fn vec2_mul(self, scalar: f32) -> Vec2 {
        self.scale(scalar)
    }
}

impl Vec2Mul<Vec2> for Vec2 {
    type Output = f32;

    fn vec2_mul(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        self.add(other)
    }
}
impl std::ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        self.sub(other)
    }
}
impl<Rhs> std::ops::Mul<Rhs> for Vec2
where
    Vec2: Vec2Mul<Rhs>,
{
    type Output = <Vec2 as Vec2Mul<Rhs>>::Output;

    fn mul(self, rhs: Rhs) -> Self::Output {
        <Vec2 as Vec2Mul<Rhs>>::vec2_mul(self, rhs)
    }
}
impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, scalar: f32) -> Vec2 {
        self.scale(1.0 / scalar)
    }
}
impl std::ops::Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}
