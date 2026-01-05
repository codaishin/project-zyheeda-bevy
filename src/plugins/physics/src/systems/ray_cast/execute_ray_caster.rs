use crate::{
	components::RayCasterArgs,
	events::RayCastInfo,
	traits::cast_ray::{
		CastRayContinuouslySorted,
		GetContinuousSortedRayCaster,
		InvalidIntersections,
	},
};
use bevy::prelude::*;
use bevy_rapier3d::plugin::ReadRapierContext;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RayCastResult {
	pub(crate) info: RayCastInfo,
}

pub(crate) fn execute_ray_caster(
	cast_ray: ReadRapierContext,
	commands: ZyheedaCommands,
	ray_casters: Query<(Entity, &RayCasterArgs)>,
) -> Result<HashMap<Entity, RayCastResult>, InvalidIntersections> {
	internal_execute_ray_caster(cast_ray, commands, ray_casters)
}

pub(crate) fn internal_execute_ray_caster<TGetRayCaster>(
	cast_ray: TGetRayCaster,
	mut commands: ZyheedaCommands,
	ray_casters: Query<(Entity, &RayCasterArgs)>,
) -> Result<HashMap<Entity, RayCastResult>, InvalidIntersections>
where
	TGetRayCaster: GetContinuousSortedRayCaster<RayCasterArgs>,
{
	let mut results = HashMap::new();
	let mut invalid_intersections = vec![];

	let Ok(ray_caster) = cast_ray.get_continuous_sorted_ray_caster() else {
		return Ok(results);
	};

	for (source, ray_caster_args) in &ray_casters {
		let hits = match ray_caster.cast_ray_continuously_sorted(ray_caster_args) {
			Ok(hits) => hits,
			Err(InvalidIntersections(invalids)) => {
				invalid_intersections.extend(invalids);
				continue;
			}
		};
		let info = RayCastInfo {
			hits,
			ray: Ray3d {
				origin: ray_caster_args.origin,
				direction: ray_caster_args.direction,
			},
			max_toi: ray_caster_args.max_toi,
		};
		results.insert(source, RayCastResult { info });
		commands.try_apply_on(&source, |mut e| {
			e.try_remove::<RayCasterArgs>();
		});
	}

	if !invalid_intersections.is_empty() {
		return Err(InvalidIntersections(invalid_intersections));
	}

	Ok(results)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{events::RayCastInfo, traits::cast_ray::SortedByTimeOfImpactAscending};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use bevy_rapier3d::math::Real;
	use common::{toi, traits::handles_physics::TimeOfImpact};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(NestedMocks)]
	struct _GetRayCaster {
		mock: Mock_GetRayCaster,
	}

	pub enum _GetRayCasterError {}

	#[automock]
	impl GetContinuousSortedRayCaster<RayCasterArgs> for _GetRayCaster {
		type TError = _GetRayCasterError;
		type TRayCaster<'a>
			= _RayCaster
		where
			Self: 'a;

		fn get_continuous_sorted_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
			self.mock.get_continuous_sorted_ray_caster()
		}
	}

	#[derive(NestedMocks)]
	pub struct _RayCaster {
		pub mock: Mock_RayCaster,
	}

	#[automock]
	impl CastRayContinuouslySorted<RayCasterArgs> for _RayCaster {
		fn cast_ray_continuously_sorted(
			&self,
			ray: &RayCasterArgs,
		) -> SortedByTimeOfImpactAscending {
			self.mock.cast_ray_continuously_sorted(ray)
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn cast_ray() -> Result<(), RunSystemError> {
		let get_ray_caster = _GetRayCaster::new().with_mock(|mock| {
			mock.expect_get_continuous_sorted_ray_caster()
				.times(1)
				.returning(|| {
					Ok(_RayCaster::new().with_mock(|mock| {
						mock.expect_cast_ray_continuously_sorted()
							.times(1)
							.with(eq(RayCasterArgs {
								origin: Vec3::ZERO,
								direction: Dir3::NEG_Y,
								max_toi: toi!(42.),
								solid: true,
								filter: default(),
							}))
							.return_const(Ok(vec![]));
					}))
				});
		});
		let mut app = setup();

		app.world_mut().spawn(RayCasterArgs {
			origin: Vec3::ZERO,
			direction: Dir3::NEG_Y,
			max_toi: toi!(42.),
			solid: true,
			filter: default(),
		});

		_ = app.world_mut().run_system_once_with(
			internal_execute_ray_caster::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;
		Ok(())
	}

	#[test]
	fn add_cast_ray_result_with_targets() -> Result<(), RunSystemError> {
		let get_ray_caster = _GetRayCaster::new().with_mock(|mock| {
			mock.expect_get_continuous_sorted_ray_caster()
				.times(1)
				.returning(|| {
					Ok(_RayCaster::new().with_mock(|mock| {
						mock.expect_cast_ray_continuously_sorted()
							.return_const(Ok(vec![
								(Entity::from_raw(42), toi!(42.)),
								(Entity::from_raw(420), toi!(420.)),
							]));
					}))
				});
		});
		let mut app = setup();
		let ray_caster = app
			.world_mut()
			.spawn(RayCasterArgs {
				origin: Vec3::ONE,
				direction: Dir3::Y,
				max_toi: toi!(Real::INFINITY),
				..default()
			})
			.id();

		let results = app.world_mut().run_system_once_with(
			internal_execute_ray_caster::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		assert_eq!(
			Ok(HashMap::from([(
				ray_caster,
				RayCastResult {
					info: RayCastInfo {
						hits: vec![
							(Entity::from_raw(42), toi!(42.)),
							(Entity::from_raw(420), toi!(420.)),
						],
						ray: Ray3d {
							origin: Vec3::ONE,
							direction: Dir3::Y
						},
						max_toi: toi!(Real::INFINITY)
					}
				}
			)])),
			results
		);
		Ok(())
	}

	#[test]
	fn return_error() -> Result<(), RunSystemError> {
		let get_ray_caster = _GetRayCaster::new().with_mock(|mock| {
			mock.expect_get_continuous_sorted_ray_caster()
				.times(1)
				.returning(|| {
					Ok(_RayCaster::new().with_mock(|mock| {
						mock.expect_cast_ray_continuously_sorted()
							.return_const(Err(InvalidIntersections(vec![Vec3::new(1., 2., 3.)])));
					}))
				});
		});
		let mut app = setup();
		app.world_mut().spawn(RayCasterArgs {
			origin: Vec3::ONE,
			direction: Dir3::Y,
			max_toi: toi!(Real::INFINITY),
			..default()
		});

		let results = app.world_mut().run_system_once_with(
			internal_execute_ray_caster::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		assert_eq!(
			Err(InvalidIntersections(vec![Vec3::new(1., 2., 3.)])),
			results
		);
		Ok(())
	}

	#[test]
	fn cast_ray_only_once() -> Result<(), RunSystemError> {
		let get_ray_caster_with_call = _GetRayCaster::new().with_mock(|mock| {
			mock.expect_get_continuous_sorted_ray_caster()
				.times(1)
				.returning(|| {
					Ok(_RayCaster::new().with_mock(|mock| {
						mock.expect_cast_ray_continuously_sorted()
							.times(1)
							.return_const(Ok(vec![]));
					}))
				});
		});
		let get_ray_caster_without_call = _GetRayCaster::new().with_mock(|mock| {
			mock.expect_get_continuous_sorted_ray_caster()
				.times(1)
				.returning(|| {
					Ok(_RayCaster::new().with_mock(|mock| {
						mock.expect_cast_ray_continuously_sorted()
							.times(0)
							.return_const(Ok(vec![]));
					}))
				});
		});
		let mut app = setup();
		app.world_mut().spawn(RayCasterArgs {
			origin: Vec3::ZERO,
			direction: Dir3::NEG_Y,
			max_toi: toi!(42.),
			solid: true,
			filter: default(),
		});

		_ = app.world_mut().run_system_once_with(
			internal_execute_ray_caster::<In<_GetRayCaster>>,
			get_ray_caster_with_call,
		)?;
		_ = app.world_mut().run_system_once_with(
			internal_execute_ray_caster::<In<_GetRayCaster>>,
			get_ray_caster_without_call,
		)?;
		Ok(())
	}

	#[test]
	fn remove_ray_caster() -> Result<(), RunSystemError> {
		let get_ray_caster = _GetRayCaster::new().with_mock(|mock| {
			mock.expect_get_continuous_sorted_ray_caster()
				.times(1)
				.returning(|| {
					Ok(_RayCaster::new().with_mock(|mock| {
						mock.expect_cast_ray_continuously_sorted()
							.return_const(Ok(vec![]));
					}))
				});
		});
		let mut app = setup();
		let ray_caster = app
			.world_mut()
			.spawn(RayCasterArgs {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Y,
				max_toi: toi!(42.),
				solid: true,
				filter: default(),
			})
			.id();

		_ = app.world_mut().run_system_once_with(
			internal_execute_ray_caster::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let ray_caster = app.world().entity(ray_caster);

		assert!(!ray_caster.contains::<RayCasterArgs>());
		Ok(())
	}
}
