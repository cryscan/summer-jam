pub trait Interpolation {
    fn lerp(self, begin: Self, end: Self) -> Self;

    fn intermediate(self, begin: Self, end: Self) -> Self;
}

impl Interpolation for f32 {
    fn lerp(self, begin: Self, end: Self) -> Self {
        begin * (1.0 - self) + end * self
    }

    fn intermediate(self, begin: Self, end: Self) -> Self {
        (self - begin) / (end - begin)
    }
}
