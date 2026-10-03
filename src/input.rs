use bevy::prelude::*;

fn mouse_input(buttons: Res<ButtonInput<MouseButton>>) {
	if buttons.just_pressed(MouseButton::Left) {
		println!("left click");
	}

	if buttons.just_pressed(MouseButton::Right) {
		println!("right click");
	}
}

fn mouse_position(
	window: Single<&Window>,
) {
	if let Some(position) = window.cursor_position() {
		println!("{position:?}");
	}
}
