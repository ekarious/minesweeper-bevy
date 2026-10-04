use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::tile::{AdjacentMines, Flag, Mine, Tile, TileState};
use crate::input::PlayerActions;
use crate::game::GameState;

pub struct BoardPlugin;

const MINE_DENSITY: usize = 15;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::new(10, 10))
            .add_systems(Startup, (setup_board, plant_mines, calculate_neighbours, debug_mines, debug_neighbours).chain())
            .add_systems(Update, handle_player_actions);
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
    let possibilities: [(isize, isize); 8] = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];

    for (index, &entity) in board.tiles.iter().enumerate() {
        if mines.get(entity).is_err() {
            continue;
        }

        let (x, y) = board.get_coordinates_from_index(index);

        for (offset_x, offset_y) in possibilities {
       		let Some(neighbour_x) = x.checked_add_signed(offset_x) else {
                continue;
            };

            let Some(neighbour_y) = y.checked_add_signed(offset_y) else {
                continue;
            };

            let Some(entity) = board.get(neighbour_x, neighbour_y) else {
                continue;
            };

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
	tiles: Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
	mut next_state: ResMut<NextState<GameState>>,
) {
    for action in actions.read() {
        match action {
            PlayerActions::Primary(entity) => {
            	primary_action(*entity, &tiles, &mut commands, &mut next_state);
            }
            PlayerActions::PrimaryDouble(entity) => {
            	primary_double_action(*entity, &tiles);
            }
            PlayerActions::Secondary(entity) => {
            	secondary_action(*entity, &tiles, &mut commands);
            }
        }
    }
}

fn primary_action(
	entity: Entity,
	tiles: &Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
	commands: &mut Commands,
	next_state: &mut NextState<GameState>,
) {
	let Ok((state, adjacent_mines, is_mine, is_flagged)) = tiles.get(entity) else {
       	return;
    };

    match state {
       	TileState::Hidden if is_flagged => {
        	// Nothing happen. Safeguard.
       	}
       	TileState::Hidden if is_mine => {
        	commands.entity(entity).insert(TileState::Visible);
      		next_state.set(GameState::GameOver);
       	}
       	TileState::Hidden => {
            commands.entity(entity).insert(TileState::Visible);

            if adjacent_mines.0 == 0 {
            	reveal_neighbors_tiles(entity);
            }
        }
        TileState::Visible => {
            // Nothing happen on an already visible tile.
        }
    }
}

fn primary_double_action(
	entity: Entity,
	tiles: &Query<(&TileState, &AdjacentMines, Has<Mine>, Has<Flag>), With<Tile>>,
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

    reveal_neighbors_tiles(entity);
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

fn reveal_neighbors_tiles(entity: Entity) {}

// Temporaire

fn debug_mines(mut mines: Query<&mut Sprite, With<Mine>>) {
    for mut sprite in &mut mines {
        sprite.color = Color::srgb(1.0, 0.0, 0.0);
    }
}

fn debug_neighbours(
	mut commands: Commands,
    board: Res<Board>,
    neighbours: Query<&AdjacentMines, Without<Mine>>,
) {
	for (_, &entity) in board.tiles.iter().enumerate() {
        let Ok(adjacent_mines) = neighbours.get(entity) else {
            continue;
        };

        if adjacent_mines.0 == 0 {
            continue;
        }

        commands.entity(entity).with_child((
            Text2d::new(adjacent_mines.0.to_string()),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
    }
}
