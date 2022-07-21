use crate::utils::Interpolation;

pub trait Damp {
    fn damp(self, target: Self, speed: f32, delta_seconds: f32) -> Self;
}

impl<T: Interpolation> Damp for T {
    fn damp(self, target: Self, speed: f32, delta_seconds: f32) -> Self {
        self.lerp(target, 1.0 - (-speed * delta_seconds).exp())
    }
}
