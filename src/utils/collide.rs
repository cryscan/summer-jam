use bevy::prelude::*;
use core::f32;

#[derive(Debug, Clone)]
pub struct Penetration {
    pub normal: Vec2,
    pub location: Vec2,
    pub depth: f32,
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub normal: Vec2,
    pub location: Vec2,
    pub near_time: f32,
    pub far_time: f32,
}

#[derive(Debug, Clone)]
pub enum Hit {
    Penetration(Penetration),
    Cast(Cast),
}

impl Hit {
    pub fn normal(&self) -> Vec2 {
        match self {
            Hit::Penetration(x) => x.normal,
            Hit::Cast(x) => x.normal,
        }
    }

    pub fn location(&self) -> Vec2 {
        match self {
            Hit::Penetration(x) => x.location,
            Hit::Cast(x) => x.location,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub previous_position: Vec2,
    pub position: Vec2,
    pub size: Vec2,
}

impl Collider {
    pub fn delta(&self) -> Vec2 {
        self.position - self.previous_position
    }
}

fn intersection(a_min: Vec2, a_max: Vec2, b_min: Vec2, b_max: Vec2) -> Vec2 {
    let min = Vec2::max(a_min, b_min);
    let max = Vec2::min(a_max, b_max);
    (min + max) / 2.0
}

/// Axis-aligned bounding box collision with "side" detection
fn penetrate(a: &Collider, b: &Collider) -> Option<Penetration> {
    let a_min = a.position - a.size / 2.0;
    let a_max = a.position + a.size / 2.0;

    let b_min = b.position - b.size / 2.0;
    let b_max = b.position + b.size / 2.0;

    // check to see if the two rectangles are intersecting
    if a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y {
        let location = intersection(a_min, a_max, b_min, b_max);

        // check to see if we hit on the left or right side
        let x = if a_min.x < b_min.x && a_max.x > b_min.x && a_max.x < b_max.x {
            Some(Penetration {
                location,
                normal: -Vec2::X,
                depth: b_min.x - a_max.x,
            })
        } else if a_min.x > b_min.x && a_min.x < b_max.x && a_max.x > b_max.x {
            Some(Penetration {
                location,
                normal: Vec2::X,
                depth: b_max.x - a_min.x,
            })
        } else {
            None
        };

        // check to see if we hit on the top or bottom side
        let y = if a_min.y < b_min.y && a_max.y > b_min.y && a_max.y < b_max.y {
            Some(Penetration {
                location,
                normal: -Vec2::Y,
                depth: b_min.y - a_max.y,
            })
        } else if a_min.y > b_min.y && a_min.y < b_max.y && a_max.y > b_max.y {
            Some(Penetration {
                location,
                normal: Vec2::Y,
                depth: b_max.y - a_min.y,
            })
        } else {
            None
        };

        // if we had an "x" and a "y" collision, pick the "primary" side using penetration depth
        match (x, y) {
            (Some(x), Some(y)) => {
                if y.depth.abs() < x.depth.abs() {
                    Some(y)
                } else {
                    Some(x)
                }
            }
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    } else {
        None
    }
}

fn cast(a: &Collider, b: &Collider) -> Option<Cast> {
    let origin = a.previous_position;
    let delta = a.delta() - b.delta();
    let padding = a.size / 2.0;

    let sign = delta.signum();
    let scale = delta.recip();

    if !scale.is_finite() {
        return None;
    }

    let near_time = (b.position - sign * (b.size / 2.0 + padding) - origin) * scale;
    let far_time = (b.position + sign * (b.size / 2.0 + padding) - origin) * scale;

    if near_time.x > far_time.y || near_time.y > far_time.x {
        return None;
    }

    let normal = if near_time.x > near_time.y {
        -sign.x * Vec2::X
    } else {
        -sign.y * Vec2::Y
    };

    let (near_time, far_time) = (near_time.x.max(near_time.y), far_time.x.min(far_time.y));
    if near_time >= 1.0 || far_time <= 0.0 {
        return None;
    }

    let a_min = a.position - a.size / 2.0 + near_time * a.delta();
    let a_max = a.position + a.size / 2.0 + near_time * a.delta();

    let b_min = b.position - b.size / 2.0 + near_time * b.delta();
    let b_max = b.position + b.size / 2.0 + near_time * b.delta();

    let location = intersection(a_min, a_max, b_min, b_max);

    Some(Cast {
        normal,
        location,
        near_time,
        far_time,
    })
}

/// Axis-aligned bounding box continuous collision.
/// Returns collision time information
pub fn collide(a: &Collider, b: &Collider) -> Option<Hit> {
    // check if already overlapped
    if let Some(x) = penetrate(a, b) {
        return Some(Hit::Penetration(x));
    }

    // threat b as stationary
    // let delta = a.delta() - b.delta();

    // don't do the test if relative velocity is slow
    // if delta.length_squared() < 1.0 {
    // return None;
    // }

    if let Some(x) = cast(a, b) {
        return Some(Hit::Cast(x));
    }

    None
}
