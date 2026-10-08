use bevy::prelude::*;

mod tile;
mod board;
mod game;
mod input;
mod ui;
mod visuals;
mod debug;

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
		.add_plugins(game::GamePlugin)
		.add_plugins(board::BoardPlugin)
		.add_plugins(input::InputPlugin)
		.add_plugins(ui::UiPlugin)
		.add_plugins(visuals::VisualsPlugin)
		.add_plugins(debug::DebugPlugin)
	    .run();
}
