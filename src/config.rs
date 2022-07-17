use bevy::prelude::*;

pub const ARENA_WIDTH: f32 = 480.0;
pub const ARENA_HEIGHT: f32 = 640.0;

pub const BACKGROUND_SHADER: &str = "shaders/background.wgsl";

pub const FONT_FIRA_MONO: &str = "fonts/FiraMono-Medium.ttf";
pub const FONT_FIRA_SANS: &str = "fonts/FiraSans-Bold.ttf";
pub const FONT_ARCADE: &str = "fonts/Arcade.ttf";
pub const FONT_KARMATIC: &str = "fonts/ka1.ttf";

pub const PLAYER_SPRITE: &str = "sprites/player.png";
pub const ENEMY_SPRITE: &str = "sprites/enemy.png";
pub const BALL_SPRITE: &str = "sprites/ball.png";
pub const HINT_SPRITE: &str = "sprites/hint.png";
pub const DEATH_SPRITE: &str = "sprites/death.png";
pub const HIT_SPRITE: &str = "sprites/hit.png";

pub const GAME_ICON: &str = "sprites/icons/icon.png";
pub const RIGHT_ICON: &str = "sprites/icons/right.png";
pub const HELP_ICON: &str = "sprites/icons/help.png";
pub const RETICLE_ICON: &str = "sprites/icons/reticle.png";
pub const WRENCH_ICON: &str = "sprites/icons/wrench.png";
pub const EXIT_ICON: &str = "sprites/icons/exit.png";

pub const PADDLE_COLOR: Color = Color::rgba(0.608, 0.678, 0.718, 0.392);
pub const SEPARATE_COLOR: Color = Color::rgba(0.5, 0.5, 0.5, 0.2);
pub const BOUNDARY_COLOR: Color = Color::NONE;
pub const HEALTH_BAR_COLOR: Color = Color::rgb(0.608, 0.678, 0.718);
pub const HEALTH_BAR_TRACKER_COLOR: Color = Color::rgb(0.851, 0.341, 0.388);
pub const HINT_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.2);

pub const FLIP_TEXT_COLORS: [Color; 2] = [Color::WHITE, Color::GOLD];

pub const MISS_AUDIO: &str = "audios/miss.flac";
pub const EXPLOSION_AUDIO: &str = "audios/explosion.flac";
pub const LOSE_AUDIO: &str = "audios/lose.flac";
pub const HIT_AUDIO: &str = "audios/hit.ogg";
pub const IMPACT_AUDIOS: [&str; 2] = ["audios/impacts/impact-1.ogg", "audios/impacts/impact-2.ogg"];

pub const BUTTON_HOVER_AUDIO: &str = "audios/button/hover.ogg";
pub const BUTTON_CLICK_AUDIO: &str = "audios/button/click.ogg";

pub const AUDIO_CHANNEL_COUNT: usize = 16;

pub const MENU_MUSIC: &str = "musics/E2M2 Myrgharok - Halls of Wandering Spirits.ogg";
pub const GAME_MUSIC: &str = "musics/E3M8 Myrgharok - Mother of All Doom.ogg";

pub const PREDICT_SIZE: usize = 100;
pub const PREDICT_TIME_STEP: f64 = 0.01;
pub const AI_TIME_STEP: f64 = 0.1;

pub const PHYSICS_REST_SPEED: f32 = 100.0;
pub const PHYSICS_TIME_STEP: f64 = 1.0 / 120.0;

pub const PADDLE_WIDTH: f32 = 96.0;
pub const PADDLE_HEIGHT: f32 = 16.0;
pub const BALL_SIZE: f32 = 16.0;

pub const PLAYER_MAX_SPEED: f32 = 2000.0;
pub const PLAYER_SENSITIVITY: f32 = 0.5;
pub const PLAYER_DAMP: f32 = 20.0;
pub const PLAYER_ASSIST_RANGE: f32 = 48.0;
pub const PLAYER_ASSIST_SPEED: f32 = 1000.0;
pub const PLAYER_ASSIST_VERTICAL_SPEED_THRESHOLD: f32 = -200.0;
pub const PLAYER_ASSIST_SPEED_THRESHOLD: f32 = 1000.0;

pub const ENEMY_MIN_SPEED: f32 = 500.0;
pub const ENEMY_MAX_SPEED: f32 = 2000.0;
pub const ENEMY_NORMAL_SPEED: f32 = 1250.0;
pub const ENEMY_BRAKE_DISTANCE: f32 = 96.0;
pub const ENEMY_DAMP: f32 = 20.0;
pub const ENEMY_HIT_RANGE_VERTICAL: f32 = 144.0;
pub const ENEMY_HIT_RANGE_HORIZONTAL: f32 = 144.0;
pub const ENEMY_HIT_SPEED_THRESHOLD: f32 = -0.0;

pub const PLAYER_BASE_BALL_COUNT: i32 = 3;
pub const ENEMY_BASE_FULL_HP: f32 = 40000.0;
pub const MAX_DAMAGE: f32 = 2000.0;

pub const BALL_GHOSTS_COUNT: usize = 16;
pub const BALL_MAX_SPEED: f32 = 3000.0;

pub const MIN_BOUNCE_AUDIO_SPEED: f32 = 500.0;
pub const MAX_BOUNCE_AUDIO_SPEED: f32 = 2500.0;
pub const MAX_BOUNCE_EFFECTS_SPEED: f32 = 2500.0;

pub const DEATH_EFFECT_SPEED: f32 = 2000.0;
pub const DEATH_EFFECT_ACCELERATION: f32 = 6000.0;
pub const HIT_EFFECT_TIME_STEP: f32 = 1.0 / 60.0;

pub const HEALTH_BAR_BIAS: f32 = 10.0;
pub const HEALTH_BAR_DAMP: f32 = 1.0;

pub const TIME_SCALE_DAMP: f32 = 100.0;
pub const GAME_OVER_TIME_SCALE_DAMP: f32 = 5.0;
