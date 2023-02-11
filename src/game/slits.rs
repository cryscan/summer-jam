use crate::{constants::*, utils::Interpolation, TimeScale};
use bevy::prelude::*;
use std::time::Duration;

#[derive(Resource)]
pub struct Slits {
    pub count: usize,
    pub state: SlitState,
}

impl Default for Slits {
    fn default() -> Self {
        Self {
            count: (ARENA_WIDTH / SLIT_BLOCK_WIDTH) as usize - 1,
            state: SlitState::Stand(0),
        }
    }
}

pub enum SlitState {
    Stand(usize),
    Move {
        previous: usize,
        next: usize,
        timer: Timer,
    },
}

#[derive(Component)]
pub struct SlitBlock {
    pub width: f32,
    pub index: usize,
}

impl SlitBlock {
    pub fn position(&self, index: usize) -> f32 {
        let offset = usize::from(self.index >= index);
        (self.index + offset) as f32 * self.width + (self.width - ARENA_WIDTH) / 2.0
    }
}

pub fn move_slit_block(slits: Res<Slits>, mut query: Query<(&mut Transform, &SlitBlock)>) {
    for (mut transform, slit) in query.iter_mut() {
        match &slits.state {
            SlitState::Stand(index) => transform.translation.x = slit.position(*index),
            SlitState::Move {
                previous,
                next,
                timer,
            } => {
                let factor = timer.elapsed_secs() / timer.duration().as_secs_f32();
                let begin = slit.position(*previous);
                let end = slit.position(*next);
                transform.translation.x = begin.lerp(end, factor);
            }
        };
    }
}

pub fn slits_system(time: Res<Time>, time_scale: Res<TimeScale>, mut slits: ResMut<Slits>) {
    let switch = match &mut slits.state {
        SlitState::Move { next, timer, .. } => timer
            .tick(Duration::from_secs_f32(time.delta_seconds() * time_scale.0))
            .just_finished()
            .then_some(*next),
        _ => None,
    };

    if let Some(index) = switch {
        slits.state = SlitState::Stand(index);
    }
}
