use crate::components::followers::Followers;
use bevy::prelude::*;

impl Followers {
	pub(crate) fn follow(
		parents: Query<(Entity, &Self)>,
		mut transforms: Query<&mut GlobalTransform>,
	) {
		for (followed, followers) in &parents {
			for follower in followers.iter() {
				let Ok((translation, rotation)) = transforms.get(followed).map(Self::unpack) else {
					continue;
				};
				let Ok(mut transform) = transforms.get_mut(follower) else {
					continue;
				};

				*transform = Transform::from_translation(translation)
					.with_rotation(rotation)
					.with_scale(transform.scale())
					.into();
			}
		}
	}

	fn unpack(transform: &GlobalTransform) -> (Vec3, Quat) {
		(transform.translation(), transform.rotation())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::followers::Follow;
	use testing::{ApproxEqual, SingleThreadedApp, assert_eq_approx};

	#[derive(Debug, PartialEq)]
	struct Characteristics {
		translation: Vec3,
		scale: Vec3,
		forward: Dir3,
	}

	impl Characteristics {
		fn new(transform: &GlobalTransform) -> Self {
			Self {
				translation: transform.translation(),
				scale: transform.scale(),
				forward: transform.forward(),
			}
		}
	}

	impl ApproxEqual<f32> for Characteristics {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			self.translation.approx_equal(&other.translation, tolerance)
				&& self.scale.approx_equal(&other.scale, tolerance)
				&& self.forward.approx_equal(&other.forward, tolerance)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Followers::follow);

		app
	}

	#[test]
	fn update_global_transform() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 2., 3.))
			.id();
		let child = app.world_mut().spawn(Follow(parent)).id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::new(&GlobalTransform::from_xyz(1., 2., 3.))),
			app.world()
				.entity(child)
				.get::<GlobalTransform>()
				.map(Characteristics::new),
			0.001,
		);
	}

	#[test]
	fn update_global_rotation() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_to(Dir3::NEG_X, Dir3::Y),
			))
			.id();
		let child = app.world_mut().spawn(Follow(parent)).id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::new(&GlobalTransform::from(
				Transform::default().looking_to(Dir3::NEG_X, Dir3::Y)
			))),
			app.world()
				.entity(child)
				.get::<GlobalTransform>()
				.map(Characteristics::new),
			0.001,
		);
	}

	#[test]
	fn do_not_update_scale() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().with_scale(Vec3::splat(2.)),
			))
			.id();
		let child = app
			.world_mut()
			.spawn((
				Follow(parent),
				GlobalTransform::from(Transform::default().with_scale(Vec3::splat(3.))),
			))
			.id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::new(&GlobalTransform::from(
				Transform::default().with_scale(Vec3::splat(3.))
			))),
			app.world()
				.entity(child)
				.get::<GlobalTransform>()
				.map(Characteristics::new),
			0.001,
		);
	}
}
