use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
pub struct FlipHorizontally(Name);

impl FlipHorizontally {
	pub fn with(name: Name) -> Self {
		Self(name)
	}

	pub(crate) fn system(
		flips: Query<&FlipHorizontally>,
		parents: Query<&Parent>,
		mut targets: Query<(Entity, &Name, &mut Transform), Added<Name>>,
	) {
		for (entity, name, mut transform) in &mut targets {
			let is_matching_flip_command = |entity| {
				let Ok(FlipHorizontally(target)) = flips.get(entity) else {
					return false;
				};
				name == target
			};

			if !parents.iter_ancestors(entity).any(is_matching_flip_command) {
				continue;
			}

			transform.rotate_y(PI);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assert_eq_approx, test_tools::utils::SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, FlipHorizontally::system);

		app
	}

	#[test]
	fn flip_if_name_matches_in_parent() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(FlipHorizontally::with(Name::from("my name")))
			.id();
		let child = app
			.world_mut()
			.spawn((
				Name::from("my name"),
				Transform::from_rotation(Quat::from_rotation_y(0.4)),
			))
			.set_parent(parent)
			.id();

		app.update();

		assert_eq_approx!(
			Some(Quat::from_rotation_y(PI + 0.4)),
			app.world()
				.entity(child)
				.get::<Transform>()
				.cloned()
				.map(|t| t.rotation),
			0.0001
		)
	}

	#[test]
	fn no_flip_if_name_does_no_match_in_parent() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(FlipHorizontally::with(Name::from("my name")))
			.id();
		let child = app
			.world_mut()
			.spawn((
				Name::from("my other name"),
				Transform::from_rotation(Quat::from_rotation_y(0.4)),
			))
			.set_parent(parent)
			.id();

		app.update();

		assert_eq!(
			Some(Quat::from_rotation_y(0.4)),
			app.world()
				.entity(child)
				.get::<Transform>()
				.cloned()
				.map(|t| t.rotation),
		)
	}

	#[test]
	fn flip_if_name_matches_in_parent_of_parent() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(FlipHorizontally::with(Name::from("my name")))
			.id();
		let in_between = app.world_mut().spawn_empty().set_parent(parent).id();
		let child = app
			.world_mut()
			.spawn((
				Name::from("my name"),
				Transform::from_rotation(Quat::from_rotation_y(0.2)),
			))
			.set_parent(in_between)
			.id();

		app.update();

		assert_eq_approx!(
			Some(Quat::from_rotation_y(PI + 0.2)),
			app.world()
				.entity(child)
				.get::<Transform>()
				.cloned()
				.map(|t| t.rotation),
			0.0001
		)
	}

	#[test]
	fn flip_only_once() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(FlipHorizontally::with(Name::from("my name")))
			.id();
		let child = app
			.world_mut()
			.spawn((
				Name::from("my name"),
				Transform::from_rotation(Quat::from_rotation_y(0.4)),
			))
			.set_parent(parent)
			.id();

		app.update();
		app.update();

		assert_eq_approx!(
			Some(Quat::from_rotation_y(PI + 0.4)),
			app.world()
				.entity(child)
				.get::<Transform>()
				.cloned()
				.map(|t| t.rotation),
			0.0001
		)
	}
}
