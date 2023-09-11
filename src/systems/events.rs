#[cfg(test)]
mod send_move_command_tests;

use crate::traits::world_position::SetWorldPositionFromRay;
use bevy::prelude::*;

pub fn get_ray(window: &Window, camera: &Camera, transform: &GlobalTransform) -> Option<Ray> {
	window
		.cursor_position()
		.and_then(|c| camera.viewport_to_world(transform, c))
}

pub fn send_move_command<TWorldPositionEvent: SetWorldPositionFromRay + Event>(
	create_event: impl Fn() -> TWorldPositionEvent,
	get_ray: impl Fn(&Window, &Camera, &GlobalTransform) -> Option<Ray>,
) -> Box<
	impl Fn(
		Res<Input<MouseButton>>,
		Query<&Window>,
		Query<(&Camera, &GlobalTransform)>,
		EventWriter<TWorldPositionEvent>,
	),
> {
	Box::new(
		move |mouse: Res<Input<MouseButton>>,
		      windows: Query<&Window>,
		      query: Query<(&Camera, &GlobalTransform)>,
		      mut event_writer: EventWriter<TWorldPositionEvent>| {
			if !mouse.just_pressed(MouseButton::Left) {
				return;
			}
			let Ok((cam, transform)) = query.get_single() else {
				return; // FIXME: Handle properly
			};
			let Ok(window) = windows.get_single() else {
				return; // FIXME: Handle properly
			};
			let Some(ray) = get_ray(window, cam, transform) else {
				return;
			};

			let mut event = create_event();
			event.set_world_position(ray);
			event_writer.send(event);
		},
	)
}
