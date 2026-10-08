use bevy::prelude::*;
use rand::seq::SliceRandom;
use std::collections::VecDeque;

use crate::tile::{AdjacentMines, Flag, Mine, Tile, TileState};
use crate::input::PlayerActions;
use crate::game::GameState;

pub struct BoardPlugin;

const MINE_DENSITY: usize = 15;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::new(10, 10))
            .add_systems(Startup, (setup_board, plant_mines, calculate_neighbours).chain())
            .add_systems(Update, handle_player_actions.after(crate::input::mouse_input));
    }
}

#[derive(Resource)]
pub struct Board {
    width: usize,
    height: usize,
    mine_count: usize,
    tile_size: f32,
    spacing: f32,
    tiles: Vec<Entity>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let mine_count = Self::calculate_mine_density(width, height);

        Self {
            width,
            height,
            mine_count,
            tile_size: 32.0,
            spacing: 2.0,
            tiles: Vec::with_capacity(width * height),
        }
    }

    pub fn index_from_coordinates(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Entity> {
        self.tiles.get(self.index_from_coordinates(x, y)?).copied()
    }

    pub fn get_coordinates_from_index(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    pub(crate) fn entities(&self) -> &[Entity] {
        &self.tiles
    }

    pub(crate) fn neighbour_indices(&self, index: usize) -> Vec<usize> {
        let (x, y) = self.get_coordinates_from_index(index);
        let mut neighbours = Vec::with_capacity(8);

        for offset_y in -1..=1 {
            for offset_x in -1..=1 {
                if offset_x == 0 && offset_y == 0 {
                    continue;
                }
                let Some(nx) = x.checked_add_signed(offset_x) else {
                    continue;
                };
                let Some(ny) = y.checked_add_signed(offset_y) else {
                    continue;
                };
                if let Some(index) = self.index_from_coordinates(nx, ny) {
                    neighbours.push(index);
                }
            }
        }

        neighbours
    }

    pub fn grid_to_world(&self, x: usize, y: usize) -> Vec2 {
        let step = self.tile_size + self.spacing;
        let center = Vec2::new(self.width as f32 - 1.0, self.height as f32 - 1.0)
            * step / 2.0;

        Vec2::new(x as f32 * step - center.x, center.y - y as f32 * step)
    }

    pub fn world_to_grid(&self, position: Vec2) -> Option<(usize, usize)> {
        if !position.is_finite() || self.width == 0 || self.height == 0 {
            return None;
        }

        let step = self.tile_size + self.spacing;
        // Measure right and down from the top-left edge of the first tile.
        let origin = self.grid_to_world(0, 0);
        let local = Vec2::new(position.x - origin.x, origin.y - position.y)
            + Vec2::splat(self.tile_size / 2.0);
        if local.x < 0.0 || local.y < 0.0 {
            return None;
        }

        let x = (local.x / step).floor() as usize;
        let y = (local.y / step).floor() as usize;
        self.index_from_coordinates(x, y)?;

        let within_tile = local - Vec2::new(x as f32, y as f32) * step;
        if within_tile.x >= self.tile_size || within_tile.y >= self.tile_size {
            return None;
        }

        Some((x, y))
    }

    // Mine density is intentionally integer-based.
    // We only need a whole number of mines and don't need floating-point precision.
    // The order of operations is important; dividing the density by 100 first
    // would truncate the result to 0.
    fn calculate_mine_density(width: usize, height: usize) -> usize {
        (width * height) * MINE_DENSITY / 100
    }
}

fn setup_board(mut commands: Commands, mut board: ResMut<Board>) {
    for index in 0..board.width * board.height {
        let (x, y) = board.get_coordinates_from_index(index);
        let position = board.grid_to_world(x, y);

        let entity = commands
            .spawn((
                Tile,
                TileState::Hidden,
                AdjacentMines(0),
                Sprite::from_color(Color::srgb(0.4, 0.4, 0.4), Vec2::splat(board.tile_size)),
                Transform::from_xyz(position.x, position.y, 0.0),
            ))
            .id();

        board.tiles.push(entity);
    }
}

// Usage of rand crate.
// We are using shuffle but partial_shuffle might become interesting later if the board grow in size.
fn plant_mines(mut commands: Commands, board: Res<Board>) {
    let mut indexes: Vec<usize> = (0..board.tiles.len()).collect();
    indexes.shuffle(&mut rand::rng());
    let mines = &indexes[0..board.mine_count];

    for &index in mines {
        let entity = board.tiles[index];
        commands.entity(entity).insert(Mine);
    }

    #[cfg(debug_assertions)]
    println!("Mine tiles {:?}", mines);
}

fn calculate_neighbours(
    board: Res<Board>,
    mines: Query<(), With<Mine>>,
    mut neighbours: Query<&mut AdjacentMines, Without<Mine>>,
) {
    for (index, &entity) in board.tiles.iter().enumerate() {
        if mines.get(entity).is_err() {
            continue;
        }

        for neighbour_index in board.neighbour_indices(index) {
            let entity = board.tiles[neighbour_index];
            let Ok(mut adjacent_mines) = neighbours.get_mut(entity) else {
                continue;
            };

            adjacent_mines.0 += 1;
        }
    }
}

fn handle_player_actions(
	mut commands: Commands,
	mut actions: MessageReader<PlayerActions>,
	board: Res<Board>,
	tiles: Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
	mut next_state: ResMut<NextState<GameState>>,
	state: Res<State<GameState>>,
) {
    if matches!(*next_state, NextState::Pending(GameState::Start)) {
        actions.clear();
        return;
    }
    for action in actions.read() {
        if matches!(state.get(), GameState::GameOver | GameState::Win | GameState::Paused)
            || matches!(*next_state, NextState::Pending(GameState::GameOver | GameState::Win | GameState::Paused)) {
            continue;
        }
        match action {
            PlayerActions::Primary(entity) => {
            	primary_action(*entity, &board, &tiles, &mut commands, &mut next_state);
            }
            PlayerActions::PrimaryDouble(entity) => {
            	primary_double_action(*entity, &board, &tiles, &mut commands, &mut next_state);
            }
            PlayerActions::Secondary(entity) => {
            	secondary_action(*entity, &tiles, &mut commands);
            }
        }
    }
}

fn primary_action(
	entity: Entity,
	board: &Board,
	tiles: &Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
	commands: &mut Commands,
	next_state: &mut NextState<GameState>,
) {
	let Ok((state, adjacent_mines, is_mine, is_flagged)) = tiles.get(entity) else {
       	return;
    };

    match state {
       	TileState::Hidden if is_flagged => {
         	#[cfg(debug_assertions)]
         	println!("Left Click: Flag on tile. Do Nothing");
       	}
       	TileState::Hidden if is_mine => {
        	commands.entity(entity).insert(TileState::Visible);
      		next_state.set(GameState::GameOver);

	       	#[cfg(debug_assertions)]
	       	println!("Left Click: Mine on tile. Game Over");
       	}
       	TileState::Hidden => {
            commands.entity(entity).insert(TileState::Visible);

            #[cfg(debug_assertions)]
	       	println!("Left Click: No sign on tile. Turn visible");

            if adjacent_mines.0 == 0 {
            	reveal_neighbors_tiles(entity, board, tiles, commands, next_state);
            }
        }
        TileState::Visible => {
            #[cfg(debug_assertions)]
	       	println!("Left Click: Tile already visible. Do nothing");
        }
    }
}

fn primary_double_action(
	entity: Entity,
	board: &Board,
	tiles: &Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
	commands: &mut Commands,
	next_state: &mut NextState<GameState>,
) {
	let Ok((state, adjacent_mines, _, _)) = tiles.get(entity) else {
       	return;
    };

    if matches!(state, TileState::Hidden) {
       	return;
    }

    if adjacent_mines.0 == 0 {
        return;
    }

    let Some(index) = board.tiles.iter().position(|&tile| tile == entity) else {
        return;
    };
    let flags = board.neighbour_indices(index).into_iter().filter(|&index| {
        tiles.get(board.tiles[index]).is_ok_and(|(_, _, _, is_flagged)| is_flagged)
    }).count();
    if flags != usize::from(adjacent_mines.0) {
        return;
    }

    #[cfg(debug_assertions)]
   	println!("-> AdjacentMine is {:?}: reveal neighbors starting...", adjacent_mines.0);

    reveal_neighbors_tiles(entity, board, tiles, commands, next_state);
}

fn secondary_action(
	entity: Entity,
	tiles: &Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
	commands: &mut Commands
) {
	let Ok((state, _, _, is_flagged)) = tiles.get(entity) else {
       	return;
    };

    if matches!(state, TileState::Visible) {
       	return;
    }

    if is_flagged {
       	commands.entity(entity).remove::<Flag>();
    } else {
       	commands.entity(entity).insert(Flag);
    }
}

fn reveal_neighbors_tiles(
    entity: Entity,
    board: &Board,
    tiles: &Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
    commands: &mut Commands,
    next_state: &mut NextState<GameState>,
) {
    let Some(start) = board.tiles.iter().position(|&tile| tile == entity) else {
        return;
    };
    let mut pending = VecDeque::from(board.neighbour_indices(start));
    let mut visited = vec![false; board.tiles.len()];
    visited[start] = true;

    while let Some(index) = pending.pop_front() {
        if visited[index] {
            continue;
        }
        // Commands are deferred, so TileState alone cannot prevent repeat visits.
        visited[index] = true;
        let entity = board.tiles[index];
        let Ok((state, adjacent_mines, is_mine, is_flagged)) = tiles.get(entity) else {
            continue;
        };
        if is_flagged || matches!(state, TileState::Visible) {
            continue;
        }

        commands.entity(entity).insert(TileState::Visible);
        if is_mine {
            next_state.set(GameState::GameOver);
            return;
        }
        if adjacent_mines.0 == 0 {
            pending.extend(board.neighbour_indices(index));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_board(width: usize, height: usize, mines: &[usize], flags: &[usize]) -> (App, Vec<Entity>) {
        let mut app = App::new();
        let mut board = Board::new(width, height);
        for index in 0..width * height {
            let count = board.neighbour_indices(index).iter()
                .filter(|index| mines.contains(index)).count() as u8;
            let mut tile = app.world_mut().spawn((Tile, TileState::Hidden, AdjacentMines(count)));
            if mines.contains(&index) {
                tile.insert(Mine);
            }
            if flags.contains(&index) {
                tile.insert(Flag);
            }
            board.tiles.push(tile.id());
        }
        let entities = board.tiles.clone();
        app.insert_resource(board)
            .insert_resource(State::new(GameState::Start))
            .insert_resource(NextState::<GameState>::default())
            .add_message::<PlayerActions>()
            .add_systems(Update, handle_player_actions);
        (app, entities)
    }

    fn send_action(app: &mut App, action: PlayerActions) {
        app.world_mut().resource_mut::<Messages<PlayerActions>>().write(action);
        app.update();
    }

    fn visible(app: &App, entity: Entity) -> bool {
        matches!(app.world().get::<TileState>(entity), Some(TileState::Visible))
    }

    #[test]
    fn empty_region_reveals_border_numbers_but_preserves_flags_and_mines() {
        let (mut app, entities) = test_board(4, 3, &[3, 7, 11], &[4]);
        send_action(&mut app, PlayerActions::Primary(entities[0]));
        for (index, &entity) in entities.iter().enumerate() {
            assert_eq!(visible(&app, entity), ![3, 4, 7, 11].contains(&index));
        }
        assert!(matches!(app.world().resource::<NextState<GameState>>(), NextState::Unchanged));
    }

    #[test]
    fn double_click_requires_matching_flags_and_reveals_safe_neighbours() {
        let (mut app, entities) = test_board(3, 3, &[0], &[]);
        app.world_mut().entity_mut(entities[4]).insert(TileState::Visible);
        send_action(&mut app, PlayerActions::PrimaryDouble(entities[4]));
        assert_eq!(entities.iter().filter(|&&entity| visible(&app, entity)).count(), 1);

        app.world_mut().entity_mut(entities[0]).insert(Flag);
        send_action(&mut app, PlayerActions::PrimaryDouble(entities[4]));
        assert!(!visible(&app, entities[0]));
        assert!(entities[1..].iter().all(|&entity| visible(&app, entity)));
    }

    #[test]
    fn double_click_with_misplaced_flag_triggers_game_over() {
        let (mut app, entities) = test_board(3, 3, &[0], &[1]);
        app.world_mut().entity_mut(entities[4]).insert(TileState::Visible);
        send_action(&mut app, PlayerActions::PrimaryDouble(entities[4]));
        assert!(visible(&app, entities[0]));
        assert!(!visible(&app, entities[1]));
        assert!(matches!(app.world().resource::<NextState<GameState>>(), NextState::Pending(GameState::GameOver)));
    }
}
