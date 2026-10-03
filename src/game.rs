use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
        .add_systems(Startup, setup_camera);
    }
}

#[derive(
    States,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
)]
pub enum GameState {
    #[default]
    Start,
    Playing,
    Paused,
    Win,
    GameOver,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
