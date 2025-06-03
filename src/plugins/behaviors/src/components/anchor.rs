use super::{Always, Once};
use crate::traits::has_filter::HasFilter;
use bevy::prelude::*;
use common::errors::{Error, Level};
use std::{any::type_name, marker::PhantomData};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct Anchor<TFilter> {
	pub(crate) target: Entity,
	phantom_data: PhantomData<TFilter>,
}

impl HasFilter for Anchor<Once> {
	type TFilter = Added<Self>;
}

impl HasFilter for Anchor<Always> {
	type TFilter = ();
}

impl<TFilter> Anchor<TFilter>
where
	Self: HasFilter + Send + Sync + 'static,
{
	pub(crate) fn to_fix_point_of(target: Entity) -> Self {
		Self {
			target,
			phantom_data: PhantomData,
		}
	}

	pub(crate) fn system(
		mut agents: Query<(&Self, &mut Transform), <Self as HasFilter>::TFilter>,
		fix_points: Query<&AnchorFixPoint>,
		transforms: Query<&GlobalTransform>,
	) -> Vec<Result<(), AnchorError>> {
		agents
			.iter_mut()
			.filter_map(|(anchor, mut anchor_transform)| {
				let Ok(AnchorFixPoint(fix_point)) = fix_points.get(anchor.target) else {
					return Some(AnchorError::NoFixPointOn(anchor.target));
				};
				let Ok(fix_point) = transforms.get(*fix_point) else {
					return Some(AnchorError::NoGlobalTransformOn(*fix_point));
				};

				let fix_point = Transform::from(*fix_point);

				anchor_transform.translation = fix_point.translation;
				anchor_transform.rotation = fix_point.rotation;

				None
			})
			.map(Err)
			.collect()
	}
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct AnchorFixPoint(Entity);

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AnchorError {
	NoFixPointOn(Entity),
	NoGlobalTransformOn(Entity),
}

impl From<AnchorError> for Error {
	fn from(error: AnchorError) -> Self {
		match error {
			AnchorError::NoFixPointOn(entity) => {
				let type_name = type_name::<AnchorFixPoint>();
				Self {
					msg: format!("{entity}: {type_name} missing"),
					lvl: Level::Error,
				}
			}
			AnchorError::NoGlobalTransformOn(entity) => {
				let type_name = type_name::<GlobalTransform>();
				Self {
					msg: format!("{entity}: {type_name} missing"),
					lvl: Level::Error,
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::SingleThreadedApp;

	struct _NotIgnored;

	#[derive(Component)]
	struct _Ignore;

	impl HasFilter for Anchor<_NotIgnored> {
		type TFilter = Without<_Ignore>;
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn copy_location_translation() -> Result<(), RunSystemError> {
		let mut app = setup();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let entity = app.world_mut().spawn(AnchorFixPoint(fix_point)).id();
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_fix_point_of(entity),
				Transform::default(),
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			app.world().entity(agent).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn copy_location_rotation() -> Result<(), RunSystemError> {
		let mut app = setup();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y),
			))
			.id();
		let entity = app.world_mut().spawn(AnchorFixPoint(fix_point)).id();
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_fix_point_of(entity),
				Transform::default(),
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Some(&Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn do_not_change_scale() -> Result<(), RunSystemError> {
		let mut app = setup();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::default()))
			.id();
		let entity = app.world_mut().spawn(AnchorFixPoint(fix_point)).id();
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_fix_point_of(entity),
				Transform::from_scale(Vec3::new(3., 4., 5.)),
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Some(&Transform::from_scale(Vec3::new(3., 4., 5.))),
			app.world().entity(agent).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn apply_filter() -> Result<(), RunSystemError> {
		let mut app = setup();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let entity = app.world_mut().spawn(AnchorFixPoint(fix_point)).id();
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_fix_point_of(entity),
				Transform::default(),
				_Ignore,
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Some(&Transform::default()),
			app.world().entity(agent).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn fix_point_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		_ = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_fix_point_of(entity),
				Transform::default(),
			))
			.id();

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(vec![Err(AnchorError::NoFixPointOn(entity))], errors);
		Ok(())
	}

	#[test]
	fn transform_missing_on_fix_point() -> Result<(), RunSystemError> {
		let mut app = setup();
		let fix_point = app.world_mut().spawn_empty().id();
		let entity = app.world_mut().spawn(AnchorFixPoint(fix_point)).id();
		_ = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_fix_point_of(entity),
				Transform::default(),
			))
			.id();

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			vec![Err(AnchorError::NoGlobalTransformOn(fix_point))],
			errors
		);
		Ok(())
	}
}
