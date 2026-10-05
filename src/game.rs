use bevy::prelude::*;

use crate::tile::{Flag, Mine, Tile, TileState};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<GameSession>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<GameAssets>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, toggle_pause.before(crate::input::mouse_input))
            .add_systems(PostUpdate, update_session);
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Start,
    Playing,
    Paused,
    Win,
    GameOver,
}

#[derive(Resource, Default)]
pub(crate) struct GameSession {
    pub(crate) elapsed: f64,
    pub(crate) started: bool,
    pub(crate) generation: u64,
}

#[derive(Resource)]
pub struct GameAssets {
    pub flag: Handle<Image>,
    pub mine: Handle<Image>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            flag: asset_server.load("images/Flag.png"),
            mine: asset_server.load("images/Mine.png"),
        }
    }
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    session: Res<GameSession>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !keys.just_pressed(KeyCode::KeyP) || !matches!(*next_state, NextState::Unchanged) {
        return;
    }

    match state.get() {
        GameState::Start | GameState::Playing => next_state.set(GameState::Paused),
        GameState::Paused => next_state.set(if session.started {
            GameState::Playing
        } else {
            GameState::Start
        }),
        GameState::Win | GameState::GameOver => {}
    }
}

pub(crate) fn update_session(
    time: Res<Time>,
    tiles: Query<(&TileState, Has<Mine>, Has<Flag>), With<Tile>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut session: ResMut<GameSession>,
) {
    if matches!(
        state.get(),
        GameState::Win | GameState::GameOver | GameState::Paused
    ) {
        return;
    }
    if matches!(
        *next_state,
        NextState::Pending(GameState::Start | GameState::GameOver | GameState::Paused)
    ) {
        return;
    }
    let mut safe = 0;
    let mut revealed = 0;
    for (tile_state, is_mine, is_flagged) in &tiles {
        session.started |= is_flagged || matches!(tile_state, TileState::Visible);
        if !is_mine {
            safe += 1;
            revealed += usize::from(matches!(tile_state, TileState::Visible));
        }
    }
    if session.started {
        session.elapsed += time.delta_secs_f64();
        if safe > 0 && revealed == safe {
            next_state.set(GameState::Win);
        } else if *state.get() == GameState::Start {
            next_state.set(GameState::Playing);
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p_pauses_timer_and_resumes_without_resetting_it() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(1));
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyP);
        app.insert_resource(time)
            .insert_resource(keys)
            .insert_resource(State::new(GameState::Playing))
            .insert_resource(NextState::<GameState>::default())
            .insert_resource(GameSession {
                elapsed: 10.0,
                started: true,
                generation: 0,
            });
        app.world_mut().spawn((Tile, TileState::Hidden));

        app.world_mut().run_system_cached(toggle_pause).unwrap();
        assert!(matches!(
            app.world().resource::<NextState<GameState>>(),
            NextState::Pending(GameState::Paused)
        ));
        app.world_mut().run_system_cached(update_session).unwrap();
        assert_eq!(app.world().resource::<GameSession>().elapsed, 10.0);

        app.insert_resource(State::new(GameState::Paused))
            .insert_resource(NextState::<GameState>::default());
        app.world_mut().run_system_cached(update_session).unwrap();
        assert_eq!(app.world().resource::<GameSession>().elapsed, 10.0);
        app.world_mut().run_system_cached(toggle_pause).unwrap();
        assert!(matches!(
            app.world().resource::<NextState<GameState>>(),
            NextState::Pending(GameState::Playing)
        ));

        app.insert_resource(State::new(GameState::Playing))
            .insert_resource(NextState::<GameState>::default());
        app.world_mut().run_system_cached(update_session).unwrap();
        assert_eq!(app.world().resource::<GameSession>().elapsed, 11.0);
    }

    #[test]
    fn p_does_not_resume_finished_games() {
        let mut app = App::new();
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyP);
        app.insert_resource(keys).init_resource::<GameSession>();
        for state in [GameState::Win, GameState::GameOver] {
            app.insert_resource(State::new(state))
                .insert_resource(NextState::<GameState>::default());
            app.world_mut().run_system_cached(toggle_pause).unwrap();
            assert!(matches!(
                app.world().resource::<NextState<GameState>>(),
                NextState::Unchanged
            ));
        }
    }
}
