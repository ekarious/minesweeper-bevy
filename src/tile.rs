use bevy::prelude::*;

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct Mine;

#[derive(Component)]
pub struct Flag;

#[derive(Component)]
pub enum TileState {
    Hidden,
    Visible,
}

#[derive(Component, Debug)]
pub struct AdjacentMines(pub u8);
