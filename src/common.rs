
#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[must_use]
pub fn min(of: i32, or: i32) -> i32 {
    of.min(or)
}

#[must_use]
pub fn max(of: i32, or: i32) -> i32 {
    of.max(or)
}

pub fn constrain<T: PartialOrd>(this: T, min: T, max: T) -> T {
    assert!(min < max);
    if this < min {
        return min;
    } else if this > max {
        return max;
    }
    this
}

#[derive(Debug, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

