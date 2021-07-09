use bevy::{math::f32, prelude::*, sprite::collide_aabb::*};

pub struct Hit {
    pub collision: Collision,
    pub near_time: f32,
    pub far_time: f32,
}

// Axis-aligned bounding box continuous collision
// Returns collision time information
pub fn collide_continuous(
    a_prev_pos: Vec3,
    a_pos: Vec3,
    a_size: Vec2,
    b_prev_pos: Vec3,
    b_pos: Vec3,
    b_size: Vec2,
) -> Option<Hit> {
    // check if already overlapped
    if let Some(collision) = collide(a_prev_pos, a_size, b_prev_pos, b_size) {
        return Some(Hit {
            collision,
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
    let scale = delta.recip();
    let sign = delta.signum();

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
        near_time,
        far_time,
    })
}
