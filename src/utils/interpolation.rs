use bevy::prelude::*;

pub trait Interpolation {
    fn lerp(self, end: Self, factor: f32) -> Self;
}

macro_rules! impl_interpolation {
    ($type:ident) => {
        impl Interpolation for $type {
            fn lerp(self, end: Self, factor: f32) -> Self {
                self * (1.0 - factor) + end * factor
            }
        }
    };
}

impl_interpolation!(f32);
impl_interpolation!(Vec2);
impl_interpolation!(Vec3);
impl_interpolation!(Vec4);

pub trait Intermediate {
    fn intermediate(self, begin: Self, end: Self) -> Self;
}

impl Intermediate for f32 {
    fn intermediate(self, begin: Self, end: Self) -> Self {
        (self - begin) / (end - begin)
    }
}
