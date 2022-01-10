use crate::utils::Interpolation;
use bevy::prelude::*;

pub trait Damp {
    fn damp(self, target: Self, speed: f32, delta_seconds: f32) -> Self;
}

impl Damp for f32 {
    fn damp(self, target: Self, speed: f32, delta_seconds: f32) -> Self {
        Interpolation::lerp(1.0 - (-speed * delta_seconds).exp(), self, target)
    }
}

impl Damp for Vec2 {
    fn damp(self, target: Self, speed: f32, delta_seconds: f32) -> Self {
        Vec2::new(
            self.x.damp(target.x, speed, delta_seconds),
            self.y.damp(target.y, speed, delta_seconds),
        )
    }
}
