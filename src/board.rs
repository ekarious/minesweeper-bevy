use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::tile::{AdjacentMines, Flag, Mine, Tile, TileState};

pub struct BoardPlugin;

const MINE_DENSITY: usize = 15;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
    	app
        .insert_resource(Board::new(10, 10))
        .add_systems(Startup, (setup_board, plant_mines).chain());
    }
}

#[derive(Resource)]
pub struct Board {
	width: usize,
	height: usize,
	mine_count: usize,
	tiles: Vec<Entity>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
    	let mine_count = Self::calculate_mine_density(width, height);

    	Self {
     		width,
	       	height,
			mine_count,
		    tiles: Vec::with_capacity(width * height),
     	}
    }

    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
    	if x < self.width && y < self.height {
     		Some(y * self.width + x)
     	} else {
          None
      	}
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Entity> {
    	self.tiles.get(self.index(x, y)?)
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
	for _ in 0..board.width * board.height {
		let entity = commands.spawn((
			Tile,
			TileState::Hidden,
			AdjacentMines(0)
		)).id();

		board.tiles.push(entity);
	}
}

// Usage of rand crate.
// We are using shuffle but partial_shuffle might become interesting later if the board grow in size.
fn plant_mines(mut commands: Commands, board: Res<Board>) {
	// Get a random number of indexes from the tiles array.
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
  	mut neighbours: Query<&mut AdjacentMines, Without<Mine>>) {
   		todo!();
}
