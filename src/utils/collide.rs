use core::f32;

use bevy::{prelude::*, sprite::collide_aabb::Collision};

#[derive(Debug)]
pub struct Hit {
    pub collision: Collision,
    pub depth: f32,
    pub near_time: f32,
    pub far_time: f32,
}

/// Axis-aligned bounding box collision with "side" detection
pub fn collide(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Option<(Collision, f32)> {
    let a_min = a_pos.truncate() - a_size / 2.0;
    let a_max = a_pos.truncate() + a_size / 2.0;

    let b_min = b_pos.truncate() - b_size / 2.0;
    let b_max = b_pos.truncate() + b_size / 2.0;

    // check to see if the two rectangles are intersecting
    if a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y {
        // check to see if we hit on the left or right side
        let (x_collision, x_depth) = if a_min.x < b_min.x && a_max.x > b_min.x && a_max.x < b_max.x
        {
            (Some(Collision::Left), b_min.x - a_max.x)
        } else if a_min.x > b_min.x && a_min.x < b_max.x && a_max.x > b_max.x {
            (Some(Collision::Right), b_max.x - a_min.x)
        } else {
            (None, 0.0)
        };

        // check to see if we hit on the top or bottom side
        let (y_collision, y_depth) = if a_min.y < b_min.y && a_max.y > b_min.y && a_max.y < b_max.y
        {
            (Some(Collision::Bottom), b_min.y - a_max.y)
        } else if a_min.y > b_min.y && a_min.y < b_max.y && a_max.y > b_max.y {
            (Some(Collision::Top), b_max.y - a_min.y)
        } else {
            (None, 0.0)
        };

        // if we had an "x" and a "y" collision, pick the "primary" side using penetration depth
        match (x_collision, y_collision) {
            (Some(x_collision), Some(y_collision)) => {
                if y_depth.abs() < x_depth.abs() {
                    Some((y_collision, y_depth))
                } else {
                    Some((x_collision, x_depth))
                }
            }
            (Some(x_collision), None) => Some((x_collision, x_depth)),
            (None, Some(y_collision)) => Some((y_collision, y_depth)),
            (None, None) => None,
        }
    } else {
        None
    }
}

/// Axis-aligned bounding box continuous collision.
/// Returns collision time information
pub fn collide_continuous(
    a_prev_pos: Vec3,
    a_pos: Vec3,
    a_size: Vec2,
    b_prev_pos: Vec3,
    b_pos: Vec3,
    b_size: Vec2,
) -> Option<Hit> {
    // check if already overlapped
    if let Some((collision, depth)) = collide(a_prev_pos, a_size, b_prev_pos, b_size) {
        return Some(Hit {
            collision,
            depth,
            near_time: 0.0,
            far_time: 1.0,
        });
    }

    // threat b as stationary
    let a_delta = (a_pos - a_prev_pos).truncate();
    let b_delta = (b_pos - b_prev_pos).truncate();
    let delta = a_delta - b_delta;

    // don't do the test if velocity is slow
    if delta.length_squared() < 1.0 {
        return None;
    }

    intersect_segment(
        b_pos.truncate(),
        b_size,
        a_pos.truncate(),
        delta,
        a_size / 2.0,
    )
}

fn intersect_segment(
    box_pos: Vec2,
    box_size: Vec2,
    origin: Vec2,
    delta: Vec2,
    padding: Vec2,
) -> Option<Hit> {
    let sign = delta.signum();
    let scale = (delta + sign * 0.01).recip();

    let near_time_x = (box_pos.x - sign.x * (box_size.x / 2.0 + padding.x) - origin.x) * scale.x;
    let near_time_y = (box_pos.y - sign.y * (box_size.y / 2.0 + padding.y) - origin.y) * scale.y;
    let far_time_x = (box_pos.x + sign.x * (box_size.x / 2.0 + padding.x) - origin.x) * scale.x;
    let far_time_y = (box_pos.y + sign.y * (box_size.y / 2.0 + padding.y) - origin.y) * scale.y;

    if near_time_x > far_time_y || near_time_y > far_time_x {
        return None;
    }

    let near_time = near_time_x.max(near_time_y);
    let far_time = far_time_x.min(far_time_y);

    if near_time >= 1.0 || far_time <= 0.0 {
        return None;
    }

    let collision = if near_time_x > near_time_y {
        match sign.x > 0.0 {
            true => Collision::Left,
            false => Collision::Right,
        }
    } else {
        match sign.y > 0.0 {
            true => Collision::Bottom,
            false => Collision::Top,
        }
    };

    Some(Hit {
        collision,
        depth: 0.0,
        near_time,
        far_time,
    })
}
