use crate::{constants::*, AppState};
use bevy::{app::AppExit, prelude::*};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetsLoading>()
            .add_system_set(SystemSet::on_enter(AppState::Loading).with_system(setup))
            .add_system_set(
                SystemSet::on_update(AppState::Loading).with_system(check_assets_loaded),
            );
    }
}

#[derive(Default, Resource, Deref, DerefMut)]
struct AssetsLoading(Vec<HandleUntyped>);

fn setup(server: Res<AssetServer>, mut loading: ResMut<AssetsLoading>) {
    loading.push(server.load_untyped(BACKGROUND_SHADER));

    loading.push(server.load_untyped(FONT_FIRA_MONO));
    loading.push(server.load_untyped(FONT_FIRA_SANS));
    loading.push(server.load_untyped(FONT_ARCADE));
    loading.push(server.load_untyped(FONT_KARMATIC));
    loading.push(server.load_untyped(FONT_BOXY));
    loading.push(server.load_untyped(FONT_ALAGARD));
    loading.push(server.load_untyped(FONT_INVASION));

    loading.push(server.load_untyped(PLAYER_SPRITE));
    loading.push(server.load_untyped(ENEMY_SPRITE));
    loading.push(server.load_untyped(BALL_SPRITE));
    loading.push(server.load_untyped(HINT_SPRITE));
    loading.push(server.load_untyped(DEATH_SPRITE));
    loading.push(server.load_untyped(HIT_SPRITE));

    loading.push(server.load_untyped(GAME_ICON));
    loading.push(server.load_untyped(RIGHT_ICON));
    loading.push(server.load_untyped(HELP_ICON));
    loading.push(server.load_untyped(RETICLE_ICON));
    loading.push(server.load_untyped(WRENCH_ICON));
    loading.push(server.load_untyped(EXIT_ICON));

    loading.push(server.load_untyped(HIT_AUDIO));
    loading.push(server.load_untyped(MISS_AUDIO));
    loading.push(server.load_untyped(EXPLOSION_AUDIO));
    loading.push(server.load_untyped(LOSE_AUDIO));

    for audio in IMPACT_AUDIOS {
        loading.push(server.load_untyped(audio));
    }

    loading.push(server.load_untyped(BUTTON_HOVER_AUDIO));
    loading.push(server.load_untyped(BUTTON_CLICK_AUDIO));

    loading.push(server.load_untyped(MENU_MUSIC));
    loading.push(server.load_untyped(GAME_MUSIC));
}

fn check_assets_loaded(
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut app_state: ResMut<State<AppState>>,
    mut exit: EventWriter<AppExit>,
) {
    use bevy::asset::LoadState;
    match server.get_group_load_state(loading.iter().map(|handle| handle.id)) {
        LoadState::Loaded => {
            info!("Assets Loaded");
            app_state.set(AppState::Menu).unwrap();
        }
        LoadState::Failed => {
            info!("Assets Loading Failed");
            exit.send(AppExit);
        }
        _ => {}
    }
}
