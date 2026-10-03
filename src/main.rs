use bevy::prelude::*;

mod tile;
mod board;
mod game;
mod input;
mod ui;

use game::GamePlugin;
use board::BoardPlugin;

fn main() {
    App::new()
	    .add_plugins(DefaultPlugins.set(WindowPlugin {
	        primary_window: Some(Window {
	            title: String::from(
	                "Minesweeper",
	            ),
	            ..Default::default()
	        }),
	        ..default()
	    }))
		.add_plugins(GamePlugin)
		.add_plugins(BoardPlugin)
	    .run();
}
