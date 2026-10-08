use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::board::Board;
use crate::debug::DebugSettings;
use crate::game::{GameSession, GameState, update_session};
use crate::tile::{AdjacentMines, Flag, Mine, Tile, TileState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, handle_controls.before(crate::input::mouse_input))
            .add_systems(PostUpdate, update_labels.after(update_session));
    }
}

#[derive(Component, Clone, Default)]
enum HudLabel {
    #[default]
    Timer,
    Score,
    Mines,
    Status,
    MineToggle,
    NeighbourToggle,
}

#[derive(Component, Clone, Copy, Default)]
enum DebugControl {
    #[default]
    NewBoard,
    Restart,
    ToggleMines,
    ToggleNeighbours,
}

fn hud_text(label: HudLabel, value: &str) -> impl Scene + use<> {
    let value = value.to_owned();
    bsn! {
        Text(value) TextFont { font_size: px(18) }
        TextColor(Color::srgb(0.92, 0.94, 0.95)) template_value(label)
    }
}

fn control(action: DebugControl, label: &str) -> impl Scene + use<> {
    let label = label.to_owned();
    bsn! {
        Button template_value(action)
        Node {
            min_width: px(110), min_height: px(36),
            padding: UiRect::axes(px(12), px(8)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BackgroundColor(Color::srgb(0.18, 0.22, 0.23))
        Children[(Text(label) TextFont { font_size: px(16) })]
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100),
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
        }
        BackgroundColor(Color::srgb(0.08, 0.10, 0.11))
        Interaction
        Children[
            (Text("Minesweeper") TextFont { font_size: px(24) }),
            (
                Node { flex_wrap: FlexWrap::Wrap, column_gap: px(24), row_gap: px(8) }
                Children[
                    hud_text(HudLabel::Timer, "Time  00:00"),
                    hud_text(HudLabel::Score, "Score  0"),
                    hud_text(HudLabel::Mines, "Mines  15"),
                    hud_text(HudLabel::Status, "Ready")
                ]
            )
        ]
    });
    commands.spawn_scene(bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: px(0), width: percent(100),
            padding: UiRect::all(px(12)),
            align_items: AlignItems::Center,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(10), row_gap: px(8),
        }
        BackgroundColor(Color::srgb(0.08, 0.10, 0.11))
        Interaction
        Children[
            (Text("Debug") TextFont { font_size: px(16) }),
            control(DebugControl::NewBoard, "New board"),
            control(DebugControl::Restart, "Restart"),
            (
                Button template_value(DebugControl::ToggleMines)
                Node {
                    min_width: px(150), min_height: px(36),
                    padding: UiRect::axes(px(12), px(8)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                }
                BackgroundColor(Color::srgb(0.18, 0.22, 0.23))
                Children[hud_text(HudLabel::MineToggle, "[ ] Show mines")]
            ),
            (
                Button template_value(DebugControl::ToggleNeighbours)
                Node {
                    min_width: px(180), min_height: px(36),
                    padding: UiRect::axes(px(12), px(8)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                }
                BackgroundColor(Color::srgb(0.18, 0.22, 0.23))
                Children[hud_text(HudLabel::NeighbourToggle, "[ ] Show neighbors")]
            )
        ]
    });
}

fn handle_controls(
    mut controls: Query<(&Interaction, &DebugControl, &mut BackgroundColor), Changed<Interaction>>,
    board: Res<Board>,
    mines: Query<(), With<Mine>>,
    mut settings: ResMut<DebugSettings>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for (interaction, control, mut color) in &mut controls {
        color.0 = match interaction {
            Interaction::Pressed => Color::srgb(0.22, 0.48, 0.35),
            Interaction::Hovered => Color::srgb(0.28, 0.33, 0.34),
            Interaction::None => Color::srgb(0.18, 0.22, 0.23),
        };
        if *interaction != Interaction::Pressed {
            continue;
        }
        if matches!(control, DebugControl::ToggleMines) {
            settings.show_mines = !settings.show_mines;
            continue;
        }
        if matches!(control, DebugControl::ToggleNeighbours) {
            settings.show_neighbours = !settings.show_neighbours;
            continue;
        }

        let entities = board.entities();
        let mut mine_indices: Vec<usize> = entities
            .iter()
            .enumerate()
            .filter_map(|(index, &entity)| mines.contains(entity).then_some(index))
            .collect();
        if matches!(control, DebugControl::NewBoard) {
            let count = mine_indices.len();
            mine_indices = (0..entities.len()).collect();
            mine_indices.shuffle(&mut rand::rng());
            mine_indices.truncate(count);
        }
        for (index, &entity) in entities.iter().enumerate() {
            let count = board
                .neighbour_indices(index)
                .iter()
                .filter(|index| mine_indices.contains(index))
                .count() as u8;
            let mut tile = commands.entity(entity);
            tile.remove::<(Flag, Mine)>()
                .insert((TileState::Hidden, AdjacentMines(count)));
            if mine_indices.contains(&index) {
                tile.insert(Mine);
            }
        }
        session.elapsed = 0.0;
        session.started = false;
        session.generation += 1;
        next_state.set(GameState::Start);
    }
}

fn update_labels(
    session: Res<GameSession>,
    settings: Res<DebugSettings>,
    state: Res<State<GameState>>,
    tiles: Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
    mut labels: Query<(&HudLabel, &mut Text)>,
) {
    let mut mine_count = 0_i32;
    let mut flags = 0_i32;
    let mut score = 0;
    for (tile_state, _, is_mine, is_flagged) in &tiles {
        mine_count += i32::from(is_mine);
        flags += i32::from(is_flagged);
        score += usize::from(!is_mine && matches!(tile_state, TileState::Visible));
    }
    let seconds = session.elapsed as u64;
    for (label, mut text) in &mut labels {
        let value = match label {
            HudLabel::Timer => format!("Time  {:02}:{:02}", seconds / 60, seconds % 60),
            HudLabel::Score => format!("Score  {score}"),
            HudLabel::Mines => format!("Mines  {}", mine_count - flags),
            HudLabel::Status => match state.get() {
                GameState::Start => "Ready",
                GameState::Playing => "Playing",
                GameState::Paused => "Paused",
                GameState::Win => "Won",
                GameState::GameOver => "Game over",
            }
            .to_owned(),
            HudLabel::MineToggle => format!(
                "[{}] Show mines",
                if settings.show_mines { "x" } else { " " }
            ),
            HudLabel::NeighbourToggle => format!(
                "[{}] Show neighbors",
                if settings.show_neighbours { "x" } else { " " }
            ),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{board::BoardPlugin, game::GamePlugin, input::PlayerActions};

    fn test_app() -> App {
        let mut app = App::new();
        app.insert_resource(crate::game::GameAssets {
            flag: default(),
            mine: default(),
        });
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            GamePlugin,
            BoardPlugin,
            crate::visuals::VisualsPlugin,
            crate::debug::DebugPlugin,
        ))
        .add_message::<PlayerActions>()
        .add_systems(Update, handle_controls);
        app.update();
        app
    }

    fn press(app: &mut App, control: DebugControl) {
        app.world_mut()
            .spawn((control, Interaction::Pressed, BackgroundColor::default()));
        app.update();
    }

    #[test]
    fn restart_preserves_mines_and_resets_tiles_and_timer() {
        let mut app = test_app();
        let entities = app.world().resource::<Board>().entities().to_vec();
        let mines: Vec<_> = entities
            .iter()
            .copied()
            .filter(|&entity| app.world().get::<Mine>(entity).is_some())
            .collect();
        app.world_mut()
            .entity_mut(entities[0])
            .insert((TileState::Visible, Flag));
        app.world_mut().resource_mut::<GameSession>().elapsed = 42.0;
        press(&mut app, DebugControl::Restart);
        assert_eq!(app.world().resource::<GameSession>().elapsed, 0.0);
        assert_eq!(app.world().resource::<GameSession>().generation, 1);
        for entity in entities {
            assert!(matches!(
                app.world().get::<TileState>(entity),
                Some(TileState::Hidden)
            ));
            assert!(app.world().get::<Flag>(entity).is_none());
            assert_eq!(
                app.world().get::<Mine>(entity).is_some(),
                mines.contains(&entity)
            );
        }
    }

    #[test]
    fn new_board_keeps_mine_count_and_recalculates_numbers() {
        let mut app = test_app();
        press(&mut app, DebugControl::NewBoard);
        let board = app.world().resource::<Board>();
        let entities = board.entities();
        assert_eq!(
            entities
                .iter()
                .filter(|&&entity| app.world().get::<Mine>(entity).is_some())
                .count(),
            15
        );
        for (index, &entity) in entities.iter().enumerate() {
            let expected = board
                .neighbour_indices(index)
                .iter()
                .filter(|&&index| app.world().get::<Mine>(entities[index]).is_some())
                .count();
            assert_eq!(
                usize::from(app.world().get::<AdjacentMines>(entity).unwrap().0),
                expected
            );
        }
    }

    #[test]
    fn mine_toggle_hides_debug_color() {
        let mut app = test_app();
        assert!(!app.world().resource::<DebugSettings>().show_mines);
        app.world_mut().resource_mut::<DebugSettings>().show_mines = true;
        press(&mut app, DebugControl::ToggleMines);
        app.update();
        assert!(!app.world().resource::<DebugSettings>().show_mines);
        let board = app.world().resource::<Board>();
        for &entity in board.entities() {
            assert_eq!(
                app.world().get::<Sprite>(entity).unwrap().color,
                Color::srgb(0.4, 0.4, 0.4)
            );
        }
    }

    #[test]
    fn pending_game_over_is_not_replaced_by_win() {
        let mut app = test_app();
        let entities = app.world().resource::<Board>().entities().to_vec();
        for entity in entities {
            app.world_mut()
                .entity_mut(entity)
                .insert(TileState::Visible);
        }
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::GameOver);
        app.world_mut().run_system_cached(update_session).unwrap();
        assert!(matches!(
            app.world().resource::<NextState<GameState>>(),
            NextState::Pending(GameState::GameOver)
        ));
    }
}
