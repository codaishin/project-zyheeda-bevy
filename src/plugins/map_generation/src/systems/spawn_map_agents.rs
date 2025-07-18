use crate::{
	components::map::{
		agents::{Enemy, Player},
		cells::{MapCells, agent::Agent},
	},
	grid_graph::grid_context::{GridContext, GridDefinition, GridDefinitionError},
	traits::{GridCellDistanceDefinition, grid_min::GridMin},
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

impl<TCell> MapCells<Agent<TCell>>
where
	TCell: GridCellDistanceDefinition + ThreadSafe,
{
	pub(crate) fn spawn_map_agents(
		mut commands: Commands,
		cells: Query<&Self>,
	) -> Result<(), Vec<GridDefinitionError>> {
		let mut errors = vec![];

		for MapCells { cells, size, .. } in &cells {
			let context = GridContext::try_from(GridDefinition {
				cell_count_x: size.x,
				cell_count_z: size.z,
				cell_distance: TCell::CELL_DISTANCE,
			});
			let Ok(context) = context else {
				errors.push(GridDefinitionError::CellCountZero);
				continue;
			};
			let min = context.grid_min();

			for ((x, z), cell) in cells.iter() {
				match cell {
					Agent::Player => {
						commands.spawn((Player, transform::<TCell>(x, z, min)));
					}
					Agent::Enemy => {
						commands.spawn((Enemy, transform::<TCell>(x, z, min)));
					}
					_ => {}
				}
			}
		}

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

fn transform<TCell>(x: &usize, z: &usize, min: Vec3) -> Transform
where
	TCell: GridCellDistanceDefinition,
{
	let translation = min + Vec3::new(*x as f32, 0., *z as f32) * TCell::CELL_DISTANCE;
	Transform::from_translation(translation)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::map::cells::Size;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_count, assert_eq_unordered};

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell;

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: f32 = 4.;
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	macro_rules! entities_with {
		($ty:ty, $app:expr) => {
			$app.world().iter_entities().filter(|e| e.contains::<$ty>())
		};
	}

	#[test]
	fn spawn_player_on_1_by_1_grid() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			size: Size { x: 1, z: 1 },
			cells: HashMap::from([((0, 0), Agent::<_Cell>::Player)]),
			..default()
		});

		_ = app
			.world_mut()
			.run_system_once(MapCells::<Agent<_Cell>>::spawn_map_agents)?;

		let [player] = assert_count!(1, entities_with!(Player, app));
		assert_eq!(
			Some(&Transform::from_xyz(0., 0., 0.)),
			player.get::<Transform>(),
		);
		Ok(())
	}

	#[test]
	fn spawn_enemy_on_1_by_1_grid() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			size: Size { x: 1, z: 1 },
			cells: HashMap::from([((0, 0), Agent::<_Cell>::Enemy)]),
			..default()
		});

		_ = app
			.world_mut()
			.run_system_once(MapCells::<Agent<_Cell>>::spawn_map_agents)?;

		let [enemy] = assert_count!(1, entities_with!(Enemy, app));
		assert_eq!(
			Some(&Transform::from_xyz(0., 0., 0.)),
			enemy.get::<Transform>(),
		);
		Ok(())
	}

	#[test]
	fn spawn_on_3_by_3_grid() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			size: Size { x: 3, z: 3 },
			cells: HashMap::from([
				((0, 0), Agent::<_Cell>::Enemy),
				((0, 1), Agent::<_Cell>::Enemy),
				((0, 2), Agent::<_Cell>::Enemy),
				((1, 0), Agent::<_Cell>::Player),
				((1, 1), Agent::<_Cell>::Enemy),
				((1, 2), Agent::<_Cell>::Enemy),
				((2, 0), Agent::<_Cell>::Enemy),
				((2, 1), Agent::<_Cell>::Enemy),
				((2, 2), Agent::<_Cell>::Enemy),
			]),
			..default()
		});

		_ = app
			.world_mut()
			.run_system_once(MapCells::<Agent<_Cell>>::spawn_map_agents)?;

		let [player] = assert_count!(1, entities_with!(Player, app));
		let enemies = assert_count!(8, entities_with!(Enemy, app));
		assert_eq!(
			Some(&Transform::from_xyz(0., 0., -4.)),
			player.get::<Transform>(),
		);
		assert_eq_unordered!(
			[
				Some(&Transform::from_xyz(-4., 0., -4.)),
				Some(&Transform::from_xyz(-4., 0., 0.)),
				Some(&Transform::from_xyz(-4., 0., 4.)),
				Some(&Transform::from_xyz(0., 0., 0.)),
				Some(&Transform::from_xyz(0., 0., 4.)),
				Some(&Transform::from_xyz(4., 0., -4.)),
				Some(&Transform::from_xyz(4., 0., 0.)),
				Some(&Transform::from_xyz(4., 0., 4.)),
			],
			enemies.map(|e| e.get::<Transform>()),
		);
		Ok(())
	}

	#[test]
	fn return_0_by_0_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(MapCells::<Agent<_Cell>> {
			size: Size { x: 0, z: 0 },
			cells: HashMap::from([]),
			..default()
		});

		let result = app
			.world_mut()
			.run_system_once(MapCells::<Agent<_Cell>>::spawn_map_agents)?;

		assert_eq!(Err(vec![GridDefinitionError::CellCountZero]), result);
		Ok(())
	}
}
