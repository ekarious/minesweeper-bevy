use bevy::prelude::*;

use crate::board::Board;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_message::<PlayerActions>()
        .add_systems(Update, mouse_input);
    }
}

// Represents the actions the player can take in the game.
// There are essentially two types of player actions: primary and secondary.
// The actual behavior depends on the entity's components
// and is not the responsibility of input.rs.
#[derive(Message)]
pub enum PlayerActions {
	Primary(Entity),
	Secondary(Entity),
}

fn mouse_input(
	buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    board: Res<Board>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut actions: MessageWriter<PlayerActions>,
) {
	let primary = buttons.just_pressed(MouseButton::Left);
    let secondary = buttons.just_pressed(MouseButton::Right);

    if !primary && !secondary {
        return;
    }

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera;
    let Ok(world_position) =
        camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    let Some((x, y)) = board.world_to_grid(world_position) else {
        return;
    };

    let Some(entity) = board.get(x, y) else {
        return;
    };

    if primary {
        actions.write(PlayerActions::Primary(entity));
    }

    if secondary {
        actions.write(PlayerActions::Secondary(entity));
    }
}
