use crate::utils::{Damp, TimeScale};
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerBase {
    pub balls: i32,
}

#[derive(Component)]
pub struct EnemyBase {
    pub full_hp: f32,
    pub hp: f32,
}

#[derive(Component)]
pub struct BallCounter;

pub fn count_ball(
    base_query: Query<&PlayerBase>,
    mut counter_query: Query<&mut Text, With<BallCounter>>,
) {
    let base = base_query.single();
    for mut counter in counter_query.iter_mut() {
        counter.sections[1].value = base.balls.to_string();
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(new, Component)]
pub struct HealthBarTracker {
    damp: f32,
    bias: f32,
    #[new(value = "100.0")]
    percent: f32,
}

pub fn health_bar(base_query: Query<&EnemyBase>, mut query: Query<&mut Style, With<HealthBar>>) {
    let base = base_query.single();
    for mut health_bar in query.iter_mut() {
        health_bar.size.width = Val::Percent(base.hp / base.full_hp * 100.0);
    }
}

pub fn health_bar_tracker(
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    base_query: Query<&EnemyBase>,
    mut query: Query<(&mut Style, &mut HealthBarTracker)>,
) {
    let base = base_query.single();
    for (mut health_bar, mut tracker) in query.iter_mut() {
        let percent_hp = base.hp / base.full_hp * 100.0;
        tracker.percent = percent_hp.max(
            (tracker.percent + tracker.bias).damp(
                percent_hp,
                tracker.damp,
                time.delta_seconds() * time_scale.0,
            ) - tracker.bias,
        );
        health_bar.size.width = Val::Percent(tracker.percent - percent_hp);
    }
}
