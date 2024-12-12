use super::{Always, Once};
use crate::traits::has_filter::HasFilter;
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct SetPositionAndRotation<T> {
	pub(crate) entity: Entity,
	phantom_data: PhantomData<T>,
}

impl HasFilter for SetPositionAndRotation<Once> {
	type TFilter = Added<SetPositionAndRotation<Once>>;
}

impl HasFilter for SetPositionAndRotation<Always> {
	type TFilter = ();
}

impl<T> SetPositionAndRotation<T>
where
	Self: HasFilter + Send + Sync + 'static,
{
	pub(crate) fn to(entity: Entity) -> Self {
		Self {
			entity,
			phantom_data: PhantomData,
		}
	}

	pub(crate) fn system(
		mut agents: Query<(&Self, &mut Transform), <Self as HasFilter>::TFilter>,
		transforms: Query<&GlobalTransform>,
	) {
		for (set_pos_rot, mut transform) in &mut agents {
			let Ok(pos_rot) = transforms.get(set_pos_rot.entity) else {
				continue;
			};
			let pos_rot = Transform::from(*pos_rot);

			transform.translation = pos_rot.translation;
			transform.rotation = pos_rot.rotation;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	struct _NotIgnored;

	#[derive(Component)]
	struct _Ignore;

	impl HasFilter for SetPositionAndRotation<_NotIgnored> {
		type TFilter = Without<_Ignore>;
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, SetPositionAndRotation::<_NotIgnored>::system);

		app
	}

	#[test]
	fn copy_location_translation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				SetPositionAndRotation::<_NotIgnored>::to(entity),
				Transform::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn copy_location_rotation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y),
			))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				SetPositionAndRotation::<_NotIgnored>::to(entity),
				Transform::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn do_not_change_scale() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::default()))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				SetPositionAndRotation::<_NotIgnored>::to(entity),
				Transform::from_scale(Vec3::new(3., 4., 5.)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_scale(Vec3::new(3., 4., 5.))),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn apply_filter() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				SetPositionAndRotation::<_NotIgnored>::to(entity),
				Transform::default(),
				_Ignore,
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::default()),
			app.world().entity(agent).get::<Transform>()
		);
	}
}
