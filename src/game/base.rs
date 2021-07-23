use crate::{states::prelude::*, utils::Damp};
use bevy::prelude::*;
use std::error::Error;

#[derive(new)]
pub struct PlayerBase {
    pub balls: i32,
}

#[derive(new)]
pub struct EnemyBase {
    full_hp: f32,
    pub hp: f32,
}

pub struct BallCounter;

pub fn ball_counter(
    base_query: Query<&PlayerBase>,
    mut counter_query: Query<&mut Text, With<BallCounter>>,
) {
    let mut closure = || -> Result<(), Box<dyn Error>> {
        let base = base_query.single()?;

        for mut counter in counter_query.iter_mut() {
            counter.sections[1].value = base.balls.to_string();
        }

        Ok(())
    };

    closure().unwrap_or_default()
}

pub struct HealthBar;

#[derive(new)]
pub struct HealthBarTracker {
    damp: f32,
    bias: f32,
    #[new(default)]
    percent: f32,
}

pub fn health_bar(
    time: Res<Time>,
    mut events: EventReader<PlayerHitEvent>,
    base_query: Query<&EnemyBase>,
    mut health_bar_query: QuerySet<(
        Query<&mut Style, With<HealthBar>>,
        Query<(&mut Style, &mut HealthBarTracker)>,
    )>,
) {
    let mut closure = || -> Result<(), Box<dyn Error>> {
        let base = base_query.single()?;

        for mut health_bar in health_bar_query.q0_mut().iter_mut() {
            health_bar.size.width = Val::Percent(base.hp / base.full_hp * 100.0);
        }

        for (mut health_bar, mut tracker) in health_bar_query.q1_mut().iter_mut() {
            tracker.percent = 0.0_f32.max(
                (tracker.percent + tracker.bias).damp(0.0, tracker.damp, time.delta_seconds())
                    - tracker.bias,
            );
            health_bar.size.width = Val::Percent(tracker.percent);
        }

        for event in events.iter() {
            for (_, mut tracker) in health_bar_query.q1_mut().iter_mut() {
                tracker.percent += event.0 / base.full_hp * 100.0;
            }
        }

        Ok(())
    };

    closure().unwrap_or_default()
}
