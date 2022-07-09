use crate::{config::*, utils::Damp, TimeScale};
use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerBase {
    pub ball_count: i32,
}

impl Default for PlayerBase {
    fn default() -> Self {
        Self {
            ball_count: PLAYER_BASE_BALL_COUNT,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemyBase {
    pub full_hp: f32,
    pub hp: f32,
}

impl Default for EnemyBase {
    fn default() -> Self {
        Self {
            full_hp: ENEMY_BASE_FULL_HP,
            hp: ENEMY_BASE_FULL_HP,
        }
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct BallCounter;

pub fn count_ball(
    base_query: Query<&PlayerBase>,
    mut counter_query: Query<&mut Text, With<BallCounter>>,
) {
    let base = base_query.single();
    for mut counter in counter_query.iter_mut() {
        counter.sections[1].value = base.ball_count.to_string();
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct HealthBar;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HealthBarTracker {
    pub damp: f32,
    pub bias: f32,
    pub percent: f32,
}

impl Default for HealthBarTracker {
    fn default() -> Self {
        Self {
            damp: HEALTH_BAR_DAMP,
            bias: HEALTH_BAR_BIAS,
            percent: 100.0,
        }
    }
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
