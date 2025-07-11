use crate::components::grid::Grid;
use bevy::{ecs::query::QueryFilter, prelude::*};
use bevy_rapier3d::plugin::ReadRapierContext;
use common::traits::{
	accessors::get::Getter,
	cast_ray::{CastRay, GetRayCaster, read_rapier_context::OnlySensors},
};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct EntityOfGrid(Entity);

impl EntityOfGrid {
	pub(crate) fn get_grid<TFilter>(
		get_ray_caster: ReadRapierContext,
		agents: Query<(Entity, &GlobalTransform), TFilter>,
		grids: Query<(), With<Grid>>,
	) -> Result<HashMap<Entity, Self>, BevyError>
	where
		TFilter: QueryFilter,
	{
		get_grid(get_ray_caster, agents, grids)
	}
}

impl Getter<Entity> for EntityOfGrid {
	fn get(&self) -> Entity {
		self.0
	}
}

fn get_grid<TGetRayCaster, TFilter>(
	get_ray_caster: TGetRayCaster,
	agents: Query<(Entity, &GlobalTransform), TFilter>,
	grids: Query<(), With<Grid>>,
) -> Result<HashMap<Entity, EntityOfGrid>, TGetRayCaster::TError>
where
	TGetRayCaster: GetRayCaster<(Ray3d, OnlySensors)>,
	TFilter: QueryFilter,
{
	let ray_caster = get_ray_caster.get_ray_caster()?;
	let grid_map = agents
		.iter()
		.filter_map(|(entity, transform)| {
			let ray = Ray3d {
				origin: transform.translation(),
				direction: Dir3::NEG_Y,
			};

			let (maybe_grid, ..) = ray_caster.cast_ray(&(ray, OnlySensors))?;

			if !grids.contains(maybe_grid) {
				return None;
			}

			Some((entity, EntityOfGrid(maybe_grid)))
		})
		.collect();

	Ok(grid_map)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::cast_ray::TimeOfImpact;
	use mockall::{mock, predicate::eq};
	use std::convert::Infallible;
	use testing::{Mock, SingleThreadedApp, simple_init};

	#[derive(Component)]
	struct _Agent;

	struct _GetRayCaster {
		ray_caster: Box<dyn Fn() -> Mock_RayCaster + 'static + Sync + Send>,
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
	fn get_grid_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let grid = app.world_mut().spawn(Grid::default()).id();
		let agent = app
			.world_mut()
			.spawn((_Agent, GlobalTransform::default()))
			.id();

		let result = app.world_mut().run_system_once_with(
			get_grid::<In<_GetRayCaster>, With<_Agent>>,
			_GetRayCaster {
				ray_caster: Box::new(move || {
					Mock_RayCaster::new_mock(|mock| {
						mock.expect_cast_ray()
							.return_const((grid, TimeOfImpact(0.)));
					})
				}),
			},
		)?;

		assert_eq!(Ok(HashMap::from([(agent, EntityOfGrid(grid))])), result);
		Ok(())
	}

	#[test]
	fn ignore_hit_when_it_has_no_grid() -> Result<(), RunSystemError> {
		let mut app = setup();
		let hit = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((_Agent, GlobalTransform::default()));

		let result = app.world_mut().run_system_once_with(
			get_grid::<In<_GetRayCaster>, With<_Agent>>,
			_GetRayCaster {
				ray_caster: Box::new(move || {
					Mock_RayCaster::new_mock(|mock| {
						mock.expect_cast_ray().return_const((hit, TimeOfImpact(0.)));
					})
				}),
			},
		)?;

		assert_eq!(Ok(HashMap::from([])), result);
		Ok(())
	}

	#[test]
	fn use_proper_ray() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(Grid::default());
		app.world_mut()
			.spawn((_Agent, GlobalTransform::from_xyz(1., 2., 3.)));

		_ = app.world_mut().run_system_once_with(
			get_grid::<In<_GetRayCaster>, With<_Agent>>,
			_GetRayCaster {
				ray_caster: Box::new(move || {
					Mock_RayCaster::new_mock(|mock| {
						mock.expect_cast_ray()
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
			},
		)?;

		Ok(())
	}
}
