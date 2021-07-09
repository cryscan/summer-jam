pub trait Interpolation {
    fn lerp(self, begin: Self, end: Self) -> Self;
}

impl Interpolation for f32 {
    fn lerp(self, begin: Self, end: Self) -> Self {
        begin * (1.0 - self) + end * self
    }
}
