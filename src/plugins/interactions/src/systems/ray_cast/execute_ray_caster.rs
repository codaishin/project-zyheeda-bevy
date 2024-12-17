use crate::{components::RayCaster, events::RayCastInfo};
use bevy::prelude::*;
use common::traits::{cast_ray::CastRayContinuouslySorted, try_remove_from::TryRemoveFrom};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RayCastResult {
	pub(crate) info: RayCastInfo,
}

pub(crate) fn execute_ray_caster<TCastRay: CastRayContinuouslySorted<RayCaster> + Component>(
	mut commands: Commands,
	ray_casters: Query<(Entity, &RayCaster)>,
	cast_ray: Query<&TCastRay>,
) -> HashMap<Entity, RayCastResult> {
	let mut results = HashMap::new();

	let Ok(cast_ray) = cast_ray.get_single() else {
		return results;
	};

	for (source, ray_caster) in &ray_casters {
		let info = RayCastInfo {
			hits: cast_ray.cast_ray_continuously_sorted(ray_caster),
			ray: Ray3d {
				origin: ray_caster.origin,
				direction: ray_caster.direction,
			},
			max_toi: ray_caster.max_toi,
		};
		results.insert(source, RayCastResult { info });
		commands.try_remove_from::<RayCaster>(source);
	}

	results
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::RayCastInfo;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use bevy_rapier3d::math::Real;
	use common::traits::{cast_ray::TimeOfImpact, nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMocks)]
	struct _CastRay {
		pub mock: Mock_CastRay,
	}

	#[automock]
	impl CastRayContinuouslySorted<RayCaster> for _CastRay {
		fn cast_ray_continuously_sorted(&self, ray: &RayCaster) -> Vec<(Entity, TimeOfImpact)> {
			self.mock.cast_ray_continuously_sorted(ray)
		}
	}

	fn setup(cast_ray: _CastRay) -> App {
		let mut app = App::new();
		app.world_mut().spawn(cast_ray);

		app
	}

	#[test]
	fn cast_ray() -> Result<(), RunSystemError> {
		let mut app = setup(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray_continuously_sorted()
				.times(1)
				.with(eq(RayCaster {
					origin: Vec3::ZERO,
					direction: Dir3::NEG_Y,
					max_toi: TimeOfImpact(42.),
					solid: true,
					filter: default(),
				}))
				.return_const(vec![]);
		}));

		app.world_mut().spawn(RayCaster {
			origin: Vec3::ZERO,
			direction: Dir3::NEG_Y,
			max_toi: TimeOfImpact(42.),
			solid: true,
			filter: default(),
		});

		app.world_mut()
			.run_system_once(execute_ray_caster::<_CastRay>)?;
		Ok(())
	}

	#[test]
	fn add_cast_ray_result_with_targets() -> Result<(), RunSystemError> {
		let mut app = setup(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray_continuously_sorted()
				.return_const(vec![
					(Entity::from_raw(42), TimeOfImpact(42.)),
					(Entity::from_raw(420), TimeOfImpact(420.)),
				]);
		}));
		let ray_caster = app
			.world_mut()
			.spawn(RayCaster {
				origin: Vec3::ONE,
				direction: Dir3::Y,
				max_toi: TimeOfImpact(Real::INFINITY),
				..default()
			})
			.id();

		let results = app
			.world_mut()
			.run_system_once(execute_ray_caster::<_CastRay>)?;

		assert_eq!(
			HashMap::from([(
				ray_caster,
				RayCastResult {
					info: RayCastInfo {
						hits: vec![
							(Entity::from_raw(42), TimeOfImpact(42.)),
							(Entity::from_raw(420), TimeOfImpact(420.)),
						],
						ray: Ray3d {
							origin: Vec3::ONE,
							direction: Dir3::Y
						},
						max_toi: TimeOfImpact(Real::INFINITY)
					}
				}
			)]),
			results
		);
		Ok(())
	}

	#[test]
	fn cast_ray_only_once() -> Result<(), RunSystemError> {
		let mut app = setup(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray_continuously_sorted()
				.times(1)
				.return_const(vec![]);
		}));
		app.world_mut().spawn(RayCaster {
			origin: Vec3::ZERO,
			direction: Dir3::NEG_Y,
			max_toi: TimeOfImpact(42.),
			solid: true,
			filter: default(),
		});

		app.world_mut()
			.run_system_once(execute_ray_caster::<_CastRay>)?;
		app.world_mut()
			.run_system_once(execute_ray_caster::<_CastRay>)?;
		Ok(())
	}

	#[test]
	fn remove_ray_caster() -> Result<(), RunSystemError> {
		let mut app = setup(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray_continuously_sorted()
				.return_const(vec![]);
		}));
		let ray_caster = app
			.world_mut()
			.spawn(RayCaster {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Y,
				max_toi: TimeOfImpact(42.),
				solid: true,
				filter: default(),
			})
			.id();

		app.world_mut()
			.run_system_once(execute_ray_caster::<_CastRay>)?;

		let ray_caster = app.world().entity(ray_caster);

		assert!(!ray_caster.contains::<RayCaster>());
		Ok(())
	}
}
