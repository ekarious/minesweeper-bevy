use bevy::prelude::*;

use crate::game::{GameState, update_session};
use crate::tile::{AdjacentMines, Flag, Mine, Tile, TileState};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (setup_tile_visuals, update_tile_visuals)
                .chain()
                .after(update_session),
        );
    }
}

#[derive(Component)]
pub(crate) struct TileGlyph;

fn setup_tile_visuals(mut commands: Commands, tiles: Query<Entity, Added<Tile>>) {
    for entity in &tiles {
        commands.entity(entity).with_child((
            TileGlyph,
            Text2d::new(""),
            TextFont {
                font_size: px(20).into(),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
    }
}

pub(crate) fn update_tile_visuals(
    game: Res<State<GameState>>,
    mut tiles: Query<
        (
            &TileState,
            &AdjacentMines,
            Has<Flag>,
            Has<Mine>,
            &mut Sprite,
        ),
        With<Tile>,
    >,
    mut glyphs: Query<(&ChildOf, &mut Text2d, &mut TextColor), With<TileGlyph>>,
) {
    let finished = matches!(game.get(), GameState::Win | GameState::GameOver);
    for (state, _, _, _, mut sprite) in &mut tiles {
        let color = if matches!(state, TileState::Visible) {
            Color::srgb(0.2, 0.2, 0.2)
        } else {
            Color::srgb(0.4, 0.4, 0.4)
        };
        if sprite.color != color {
            sprite.color = color;
        }
    }
    for (parent, mut text, mut color) in &mut glyphs {
        let Ok((state, count, is_flagged, is_mine, _)) = tiles.get(parent.parent()) else {
            continue;
        };
        let visible = matches!(state, TileState::Visible);
        let (value, tint) = if is_mine && (finished || visible) {
            ("M".to_owned(), Color::srgb(1.0, 0.7, 0.7))
        } else if is_flagged {
            ("F".to_owned(), Color::srgb(0.0, 1.0, 0.0))
        } else if visible && count.0 > 0 {
            (count.0.to_string(), Color::WHITE)
        } else {
            (String::new(), Color::WHITE)
        };
        if text.0 != value {
            text.0 = value;
        }
        if color.0 != tint {
            color.0 = tint;
        }
    }
}
