use crate::components::{get_grid::GetGrid, grid::Grid};
use bevy::{ecs::query::QueryFilter, prelude::*};
use bevy_rapier3d::prelude::*;
use common::traits::cast_ray::{CastRay, GetRayCaster, read_rapier_context::OnlySensors};

impl GetGrid {
	pub(crate) fn update<TFilter>(
		rapier_context: ReadRapierContext,
		entities: Query<(&mut Self, &GlobalTransform), TFilter>,
		grids: Query<&Grid>,
	) where
		TFilter: QueryFilter,
	{
		set_grid_entity(rapier_context, entities, grids);
	}
}

fn set_grid_entity<TFilter, TGetRayCaster>(
	get_ray_caster: TGetRayCaster,
	mut entities: Query<(&mut GetGrid, &GlobalTransform), TFilter>,
	grids: Query<&Grid>,
) where
	TGetRayCaster: GetRayCaster<(Ray3d, OnlySensors)>,
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

		*get_grid = match ray_caster.cast_ray(&(ray, OnlySensors)) {
			Some((entity, ..)) if grids.contains(entity) => GetGrid(Some(entity)),
			_ => GetGrid(None),
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::cast_ray::TimeOfImpact;
	use mockall::{mock, predicate::eq};
	use std::convert::Infallible;
	use testing::{Mock, SingleThreadedApp, simple_init};

	struct _GetRayCaster {
		ray_caster: Box<dyn Fn() -> Mock_RayCaster>,
	}

	impl GetRayCaster<(Ray3d, OnlySensors)> for _GetRayCaster {
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
		impl CastRay<(Ray3d, OnlySensors)> for _RayCaster {
			fn cast_ray(&self, ray_data: &(Ray3d, OnlySensors)) -> Option<(Entity, TimeOfImpact)> {
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
		let grid = app.world_mut().spawn(Grid::default()).id();
		let entity = app
			.world_mut()
			.spawn((GetGrid::default(), GlobalTransform::default()))
			.id();
		let get_ray_caster = _GetRayCaster {
			ray_caster: Box::new(move || {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray()
						.return_const((grid, TimeOfImpact(0.)));
				})
			}),
		};

		app.world_mut()
			.run_system_once_with(set_grid_entity::<(), In<_GetRayCaster>>, get_ray_caster)?;

		assert_eq!(
			Some(&GetGrid(Some(grid))),
			app.world().entity(entity).get::<GetGrid>(),
		);
		Ok(())
	}

	#[test]
	fn set_grid_to_none_if_no_hit() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GetGrid(Some(Entity::from_raw(42))),
				GlobalTransform::default(),
			))
			.id();
		let get_ray_caster = _GetRayCaster {
			ray_caster: Box::new(move || {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray().return_const(None);
				})
			}),
		};

		app.world_mut()
			.run_system_once_with(set_grid_entity::<(), In<_GetRayCaster>>, get_ray_caster)?;

		assert_eq!(
			Some(&GetGrid(None)),
			app.world().entity(entity).get::<GetGrid>(),
		);
		Ok(())
	}

	#[test]
	fn set_grid_to_none_if_target_had_no_grid() -> Result<(), RunSystemError> {
		let mut app = setup();
		let grid = app.world_mut().spawn_empty().id();
		let entity = app
			.world_mut()
			.spawn((
				GetGrid(Some(Entity::from_raw(42))),
				GlobalTransform::default(),
			))
			.id();
		let get_ray_caster = _GetRayCaster {
			ray_caster: Box::new(move || {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray()
						.return_const((grid, TimeOfImpact(0.)));
				})
			}),
		};

		app.world_mut()
			.run_system_once_with(set_grid_entity::<(), In<_GetRayCaster>>, get_ray_caster)?;

		assert_eq!(
			Some(&GetGrid(None)),
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
			ray_caster: Box::new(|| {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray()
						.times(1)
						.with(eq((
							Ray3d {
								origin: Vec3::new(1., 2., 3.),
								direction: Dir3::NEG_Y,
							},
							OnlySensors,
						)))
						.return_const(None);
				})
			}),
		};

		app.world_mut()
			.run_system_once_with(set_grid_entity::<(), In<_GetRayCaster>>, get_ray_caster)
	}

	#[test]
	fn apply_filter() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct Ignore;

		let mut app = setup();
		let grid = app.world_mut().spawn(Grid::default()).id();
		let entities = [
			app.world_mut()
				.spawn((GetGrid::default(), GlobalTransform::default(), Ignore))
				.id(),
			app.world_mut()
				.spawn((GetGrid::default(), GlobalTransform::default()))
				.id(),
		];
		let get_ray_caster = _GetRayCaster {
			ray_caster: Box::new(move || {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray()
						.return_const((grid, TimeOfImpact(0.)));
				})
			}),
		};

		app.world_mut().run_system_once_with(
			set_grid_entity::<Without<Ignore>, In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		assert_eq!(
			[Some(&GetGrid(None)), Some(&GetGrid(Some(grid)))],
			app.world().entity(entities).map(|e| e.get::<GetGrid>()),
		);
		Ok(())
	}
}
