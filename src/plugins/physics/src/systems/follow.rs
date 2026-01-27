use crate::components::followers::{Follow, FollowTransform, Followers};
use bevy::prelude::*;
use common::errors::{ErrorData, Level};

impl Followers {
	pub(crate) fn follow(
		followed: Query<(Entity, &Self, &Transform), TransformOrFollowersChanged>,
		mut global_transforms: Query<&mut GlobalTransform>,
		follower_transforms: Query<&FollowTransform>,
		children_entities: Query<(), With<ChildOf>>,
		follower_entities: Query<(), With<Follow>>,
	) -> Result<(), Vec<FollowError>> {
		let mut errors = vec![];

		for (followed, followers, followed_transform) in &followed {
			if children_entities.contains(followed) {
				errors.push(FollowError::IsChild(followed));
				continue;
			}
			if follower_entities.contains(followed) {
				errors.push(FollowError::IsFollower(followed));
				continue;
			}

			for follower in followers.iter() {
				let Ok(mut follower_global_transform) = global_transforms.get_mut(follower) else {
					continue;
				};
				let follower_transform = follower_transforms
					.get(follower)
					.map(Self::compute_transform(followed_transform))
					.unwrap_or_else(|_| *followed_transform);

				*follower_global_transform = GlobalTransform::from(
					follower_transform.with_scale(follower_global_transform.scale()),
				);
			}
		}

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}

	fn compute_transform(followed: &Transform) -> impl Fn(&FollowTransform) -> Transform {
		move |follower: &FollowTransform| Transform {
			translation: followed.translation + followed.rotation * follower.translation,
			rotation: followed.rotation * follower.rotation,
			..default()
		}
	}
}

type TransformOrFollowersChanged = Or<(Changed<Transform>, Changed<Followers>)>;

#[derive(Debug, PartialEq)]
pub(crate) enum FollowError {
	IsChild(Entity),
	IsFollower(Entity),
}

impl ErrorData for FollowError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Follow Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		match self {
			FollowError::IsChild(entity) => {
				format!("{:?}: followed is a child, but must not be a child", entity)
			}
			FollowError::IsFollower(entity) => {
				format!(
					"{:?}: followed is a follower, but must not be a follower",
					entity
				)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::f32::consts::PI;

	use super::*;
	use crate::components::followers::Follow;
	use testing::{ApproxEqual, IsChanged, SingleThreadedApp, assert_eq_approx};

	#[derive(Debug, PartialEq)]
	struct Characteristics {
		translation: Vec3,
		scale: Vec3,
		forward: Dir3,
	}

	impl From<&GlobalTransform> for Characteristics {
		fn from(transform: &GlobalTransform) -> Self {
			Self {
				translation: transform.translation(),
				scale: transform.scale(),
				forward: transform.forward(),
			}
		}
	}

	impl From<GlobalTransform> for Characteristics {
		fn from(transform: GlobalTransform) -> Self {
			Self::from(&transform)
		}
	}

	impl ApproxEqual<f32> for Characteristics {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			self.translation.approx_equal(&other.translation, tolerance)
				&& self.scale.approx_equal(&other.scale, tolerance)
				&& self.forward.approx_equal(&other.forward, tolerance)
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct SystemResult(Result<(), Vec<FollowError>>);

	impl SystemResult {
		fn update(In(result): In<Result<(), Vec<FollowError>>>, mut commands: Commands) {
			commands.insert_resource(SystemResult(result));
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				Followers::follow.pipe(SystemResult::update),
				IsChanged::<GlobalTransform>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn update_global_transform() {
		let mut app = setup();
		let followed = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
		let follower = app.world_mut().spawn(Follow(followed)).id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::from(GlobalTransform::from_xyz(1., 2., 3.))),
			app.world()
				.entity(follower)
				.get::<GlobalTransform>()
				.map(Characteristics::from),
			0.001,
		);
	}

	#[test]
	fn update_global_rotation() {
		let mut app = setup();
		let followed = app
			.world_mut()
			.spawn(Transform::default().looking_to(Dir3::NEG_X, Dir3::Y))
			.id();
		let follower = app.world_mut().spawn(Follow(followed)).id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::from(GlobalTransform::from(
				Transform::default().looking_to(Dir3::NEG_X, Dir3::Y)
			))),
			app.world()
				.entity(follower)
				.get::<GlobalTransform>()
				.map(Characteristics::from),
			0.001,
		);
	}

	#[test]
	fn do_not_update_scale() {
		let mut app = setup();
		let followed = app
			.world_mut()
			.spawn(Transform::default().with_scale(Vec3::splat(2.)))
			.id();
		let follower = app
			.world_mut()
			.spawn((
				Follow(followed),
				GlobalTransform::from(Transform::default().with_scale(Vec3::splat(3.))),
			))
			.id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::from(GlobalTransform::from(
				Transform::default().with_scale(Vec3::splat(3.))
			))),
			app.world()
				.entity(follower)
				.get::<GlobalTransform>()
				.map(Characteristics::from),
			0.001,
		);
	}

	#[test]
	fn update_global_transform_with_follower_translation() {
		let mut app = setup();
		let followed = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
		let follower = app
			.world_mut()
			.spawn((
				Follow(followed),
				FollowTransform {
					translation: Vec3::new(3., 4., 5.),
					..default()
				},
			))
			.id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::from(GlobalTransform::from_xyz(4., 6., 8.))),
			app.world()
				.entity(follower)
				.get::<GlobalTransform>()
				.map(Characteristics::from),
			0.001,
		);
	}

	#[test]
	fn update_global_transform_with_follower_translation_based_on_followed_rotation() {
		let mut app = setup();
		let followed = app
			.world_mut()
			.spawn(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y))
			.id();
		let follower = app
			.world_mut()
			.spawn((
				Follow(followed),
				FollowTransform {
					translation: Vec3::new(3., 4., 5.),
					..default()
				},
			))
			.id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::from(GlobalTransform::from(
				Transform::from_xyz(-4., 6., 6.).looking_to(Dir3::X, Dir3::Y)
			))),
			app.world()
				.entity(follower)
				.get::<GlobalTransform>()
				.map(Characteristics::from),
			0.001,
		);
	}

	#[test]
	fn update_global_transform_with_follower_rotation() {
		let mut app = setup();
		let followed = app.world_mut().spawn(Transform::default()).id();
		let follower = app
			.world_mut()
			.spawn((
				Follow(followed),
				FollowTransform {
					rotation: Quat::from_rotation_y(PI / 2.),
					..default()
				},
			))
			.id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::from(GlobalTransform::from(
				Transform::from_rotation(Quat::from_rotation_y(PI / 2.))
			))),
			app.world()
				.entity(follower)
				.get::<GlobalTransform>()
				.map(Characteristics::from),
			0.001,
		);
	}

	#[test]
	fn update_global_transform_with_follower_rotation_based_on_followed_rotation() {
		let mut app = setup();
		let followed = app
			.world_mut()
			.spawn(Transform::from_rotation(Quat::from_rotation_y(PI / 2.)))
			.id();
		let follower = app
			.world_mut()
			.spawn((
				Follow(followed),
				FollowTransform {
					rotation: Quat::from_rotation_y(PI / 2.),
					..default()
				},
			))
			.id();

		app.update();

		assert_eq_approx!(
			Some(Characteristics::from(GlobalTransform::from(
				Transform::from_rotation(Quat::from_rotation_y(PI))
			))),
			app.world()
				.entity(follower)
				.get::<GlobalTransform>()
				.map(Characteristics::from),
			0.001,
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let followed = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
		let follower = app.world_mut().spawn(Follow(followed)).id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(follower)
				.get::<IsChanged<GlobalTransform>>()
		);
	}

	#[test]
	fn act_again_on_followed_transform_change() {
		let mut app = setup();
		let followed = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
		let follower = app.world_mut().spawn(Follow(followed)).id();

		app.update();
		app.world_mut()
			.entity_mut(followed)
			.get_mut::<Transform>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world()
				.entity(follower)
				.get::<IsChanged<GlobalTransform>>()
		);
	}

	#[test]
	fn act_again_when_follower_added() {
		let mut app = setup();
		let followed = app
			.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Followers::default()))
			.id();

		app.update();
		let follower = app.world_mut().spawn(Follow(followed)).id();
		app.update();

		assert_eq_approx!(
			Some(Characteristics::from(GlobalTransform::from_xyz(1., 2., 3.))),
			app.world()
				.entity(follower)
				.get::<GlobalTransform>()
				.map(Characteristics::from),
			0.001,
		);
	}

	#[test]
	fn return_error_when_followed_has_parent() {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		let followed = app
			.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), ChildOf(parent)))
			.id();
		let follower = app.world_mut().spawn(Follow(followed)).id();

		app.update();

		assert_eq!(
			(
				Some(&GlobalTransform::default()),
				&SystemResult(Err(vec![FollowError::IsChild(followed)]))
			),
			(
				app.world().entity(follower).get::<GlobalTransform>(),
				app.world().resource::<SystemResult>()
			)
		);
	}

	#[test]
	fn return_error_when_followed_is_nested() {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		let followed = app
			.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Follow(parent)))
			.id();
		let follower = app.world_mut().spawn(Follow(followed)).id();

		app.update();

		assert_eq!(
			(
				Some(&GlobalTransform::default()),
				&SystemResult(Err(vec![FollowError::IsFollower(followed)]))
			),
			(
				app.world().entity(follower).get::<GlobalTransform>(),
				app.world().resource::<SystemResult>()
			)
		);
	}

	#[test]
	fn return_ok() {
		let mut app = setup();
		let followed = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
		app.world_mut().spawn(Follow(followed));

		app.update();

		assert_eq!(
			&SystemResult(Ok(())),
			app.world().resource::<SystemResult>(),
		);
	}
}
