use crate::{states::game::PlayerHitEvent, utils::Damp};
use bevy::prelude::*;
use std::error::Error;

#[derive(new)]
pub struct PlayerBase {
    pub lives: i32,
}

#[derive(new)]
pub struct EnemyBase {
    full_hp: f32,
    pub hp: f32,
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
    let mut resolve = || -> Result<(), Box<dyn Error>> {
        let base = base_query.single()?;

        let mut health_bar = health_bar_query.q0_mut().single_mut()?;
        health_bar.size.width = Val::Percent(base.hp / base.full_hp * 100.0);

        let (mut health_bar, mut tracker) = health_bar_query.q1_mut().single_mut()?;
        tracker.percent = 0.0_f32.max(
            (tracker.percent + tracker.bias).damp(0.0, tracker.damp, time.delta_seconds())
                - tracker.bias,
        );
        health_bar.size.width = Val::Percent(tracker.percent);

        for event in events.iter() {
            tracker.percent += event.0 / base.full_hp * 100.0;
        }

        Ok(())
    };

    resolve().unwrap_or_default()
}
