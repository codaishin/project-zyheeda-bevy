use crate::components::get_grid::GetGrid;
use bevy::{ecs::query::QueryFilter, prelude::*};
use bevy_rapier3d::prelude::*;
use common::traits::cast_ray::{CastRay, GetRayCaster};

impl GetGrid {
	pub(crate) fn update<TFilter>(
		rapier_context: ReadRapierContext,
		entities: Query<(&mut Self, &GlobalTransform), TFilter>,
	) where
		TFilter: QueryFilter,
	{
		set_grid_entity(rapier_context, entities);
	}
}

fn set_grid_entity<TFilter, TGetRayCaster>(
	get_ray_caster: TGetRayCaster,
	mut entities: Query<(&mut GetGrid, &GlobalTransform), TFilter>,
) where
	TGetRayCaster: GetRayCaster<Ray3d>,
	TFilter: QueryFilter,
{
	let Ok(ray_caster) = get_ray_caster.get_ray_caster() else {
		return;
	};

	for (mut get_grid, transform) in &mut entities {
		let ray = Ray3d {
			origin: transform.translation(),
			direction: Dir3::NEG_Y,
		};

		let Some((entity, ..)) = ray_caster.cast_ray(&ray) else {
			continue;
		};

		*get_grid = GetGrid(Some(entity));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		simple_init,
		test_tools::utils::SingleThreadedApp,
		traits::{
			cast_ray::{CastRay, TimeOfImpact},
			mock::Mock,
		},
	};
	use mockall::{mock, predicate::eq};
	use std::convert::Infallible;

	struct _GetRayCaster {
		ray_caster: fn() -> Mock_RayCaster,
	}

	impl GetRayCaster<Ray3d> for _GetRayCaster {
		type TError = Infallible;

		type TRayCaster<'a>
			= Mock_RayCaster
		where
			Self: 'a;

		fn get_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
			Ok((self.ray_caster)())
		}
	}

	mock! {
		_RayCaster {}
		impl CastRay<Ray3d> for _RayCaster {
			fn cast_ray(&self, ray_data: &Ray3d) -> Option<(Entity, TimeOfImpact)> {
				None
			}
		}
	}

	simple_init!(Mock_RayCaster);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_grid() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((GetGrid::default(), GlobalTransform::default()))
			.id();
		let get_ray_caster = _GetRayCaster {
			ray_caster: || {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray()
						.return_const((Entity::from_raw(42), TimeOfImpact(0.)));
				})
			},
		};

		app.world_mut()
			.run_system_once_with(set_grid_entity::<(), In<_GetRayCaster>>, get_ray_caster)?;

		assert_eq!(
			Some(&GetGrid(Some(Entity::from_raw(42)))),
			app.world().entity(entity).get::<GetGrid>(),
		);
		Ok(())
	}

	#[test]
	fn use_proper_ray() -> Result<(), RunSystemError> {
		let mut app = setup();
		let transform = GlobalTransform::from_xyz(1., 2., 3.);
		app.world_mut().spawn((GetGrid::default(), transform));

		let get_ray_caster = _GetRayCaster {
			ray_caster: || {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray()
						.times(1)
						.with(eq(Ray3d {
							origin: Vec3::new(1., 2., 3.),
							direction: Dir3::NEG_Y,
						}))
						.return_const(None);
				})
			},
		};

		app.world_mut()
			.run_system_once_with(set_grid_entity::<(), In<_GetRayCaster>>, get_ray_caster)
	}

	#[test]
	fn apply_filter() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct Ignore;

		let mut app = setup();
		let entities = [
			app.world_mut()
				.spawn((GetGrid::default(), GlobalTransform::default(), Ignore))
				.id(),
			app.world_mut()
				.spawn((GetGrid::default(), GlobalTransform::default()))
				.id(),
		];
		let get_ray_caster = _GetRayCaster {
			ray_caster: || {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray()
						.return_const((Entity::from_raw(42), TimeOfImpact(0.)));
				})
			},
		};

		app.world_mut().run_system_once_with(
			set_grid_entity::<Without<Ignore>, In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		assert_eq!(
			[
				Some(&GetGrid(None)),
				Some(&GetGrid(Some(Entity::from_raw(42)))),
			],
			app.world().entity(entities).map(|e| e.get::<GetGrid>()),
		);
		Ok(())
	}
}
