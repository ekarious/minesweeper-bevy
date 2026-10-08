use bevy::prelude::*;

use crate::game::GameState;
use crate::tile::{AdjacentMines, Flag, Mine, Tile, TileState};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugSettings>().add_systems(
            PostUpdate,
            (setup_debug_visuals, update_tile_visuals_debug)
                .chain()
                .after(crate::visuals::update_tile_visuals),
        );
    }
}

#[derive(Resource, Default)]
pub(crate) struct DebugSettings {
    pub show_mines: bool,
    pub show_neighbours: bool,
}

#[derive(Component)]
struct DebugGlyph;

fn setup_debug_visuals(mut commands: Commands, tiles: Query<Entity, Added<Tile>>) {
    for entity in &tiles {
        commands.entity(entity).with_child((
            DebugGlyph,
            Text2d::new(""),
            TextFont {
                font_size: px(20).into(),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));
    }
}

fn update_tile_visuals_debug(
    settings: Res<DebugSettings>,
    game: Res<State<GameState>>,
    tiles: Query<(&TileState, &AdjacentMines, Has<Flag>, Has<Mine>), With<Tile>>,
    mut glyphs: Query<(&ChildOf, &mut Text2d, &mut TextColor), With<DebugGlyph>>,
) {
    let finished = matches!(game.get(), GameState::Win | GameState::GameOver);
    for (parent, mut text, mut color) in &mut glyphs {
        let Ok((state, count, is_flagged, is_mine)) = tiles.get(parent.parent()) else {
            continue;
        };
        let hidden = matches!(state, TileState::Hidden) && !is_flagged;
        let (value, tint) = if hidden && is_mine && !finished && settings.show_mines {
            ("M".to_owned(), Color::srgb(1.0, 0.7, 0.7))
        } else if hidden && !is_mine && count.0 > 0 && settings.show_neighbours {
            (count.0.to_string(), Color::srgb(0.6, 0.8, 1.0))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visuals::{TileGlyph, VisualsPlugin};

    fn test_tile(is_mine: bool, count: u8) -> (App, Entity) {
        let mut app = App::new();
        app.insert_resource(State::new(GameState::Playing))
            .add_plugins((VisualsPlugin, DebugPlugin));
        let mut tile = app.world_mut().spawn((
            Tile,
            TileState::Hidden,
            AdjacentMines(count),
            Sprite::from_color(Color::srgb(0.4, 0.4, 0.4), Vec2::splat(32.0)),
        ));
        if is_mine {
            tile.insert(Mine);
        }
        let entity = tile.id();
        app.update();
        (app, entity)
    }

    fn glyph<T: Component>(app: &App, tile: Entity) -> (&str, Color) {
        let children = app.world().get::<Children>(tile).unwrap();
        let entity = children
            .iter()
            .find(|&entity| app.world().get::<T>(entity).is_some())
            .unwrap();
        (
            app.world().get::<Text2d>(entity).unwrap().0.as_str(),
            app.world().get::<TextColor>(entity).unwrap().0,
        )
    }

    #[test]
    fn mine_debug_toggle_does_not_change_tile_state_or_normal_flags() {
        let (mut app, tile) = test_tile(true, 0);
        assert_eq!(glyph::<TileGlyph>(&app, tile).0, "");
        assert_eq!(glyph::<DebugGlyph>(&app, tile).0, "");
        app.world_mut().resource_mut::<DebugSettings>().show_mines = true;
        app.update();
        assert_eq!(
            glyph::<DebugGlyph>(&app, tile),
            ("M", Color::srgb(1.0, 0.7, 0.7))
        );
        app.world_mut().resource_mut::<DebugSettings>().show_mines = false;
        app.world_mut().entity_mut(tile).insert(Flag);
        app.update();
        assert_eq!(
            glyph::<TileGlyph>(&app, tile),
            ("F", Color::srgb(0.0, 1.0, 0.0))
        );
        assert_eq!(glyph::<DebugGlyph>(&app, tile).0, "");
        assert!(matches!(
            app.world().get::<TileState>(tile),
            Some(TileState::Hidden)
        ));
        app.world_mut().entity_mut(tile).remove::<Flag>();
        app.update();
        assert_eq!(glyph::<TileGlyph>(&app, tile).0, "");
    }

    #[test]
    fn neighbour_debug_toggle_leaves_revealed_numbers_visible() {
        let (mut app, tile) = test_tile(false, 3);
        app.world_mut()
            .resource_mut::<DebugSettings>()
            .show_neighbours = true;
        app.update();
        assert_eq!(glyph::<DebugGlyph>(&app, tile).0, "3");
        assert_eq!(glyph::<TileGlyph>(&app, tile).0, "");
        app.world_mut()
            .resource_mut::<DebugSettings>()
            .show_neighbours = false;
        app.update();
        assert_eq!(glyph::<DebugGlyph>(&app, tile).0, "");
        app.world_mut().entity_mut(tile).insert(TileState::Visible);
        app.update();
        assert_eq!(glyph::<TileGlyph>(&app, tile).0, "3");
        assert_eq!(glyph::<DebugGlyph>(&app, tile).0, "");
    }

    #[test]
    fn finished_game_shows_mines_without_debug_or_revealing_tile_state() {
        let (mut app, tile) = test_tile(true, 0);
        for state in [GameState::GameOver, GameState::Win] {
            app.insert_resource(State::new(state));
            app.update();
            assert_eq!(glyph::<TileGlyph>(&app, tile).0, "M");
            assert_eq!(glyph::<DebugGlyph>(&app, tile).0, "");
            assert!(matches!(
                app.world().get::<TileState>(tile),
                Some(TileState::Hidden)
            ));
        }
    }
}
