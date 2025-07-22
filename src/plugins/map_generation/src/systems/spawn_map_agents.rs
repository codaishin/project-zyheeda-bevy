use crate::{
	components::{
		map::{
			agents::{AgentsLoaded, Enemy, Player},
			cells::{MapCells, agent::Agent},
		},
		map_agents::AgentOfPersistentMap,
	},
	grid_graph::grid_context::GridContext,
	traits::{GridCellDistanceDefinition, grid_min::GridMin},
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn},
};

impl<TCell> MapCells<Agent<TCell>>
where
	TCell: GridCellDistanceDefinition + ThreadSafe,
{
	pub(crate) fn spawn_map_agents(
		mut commands: Commands,
		cells: Query<(Entity, &PersistentEntity, &Self)>,
	) {
		for (entity, persistent_entity, MapCells { cells, size, .. }) in &cells {
			let context = GridContext {
				cell_count_x: size.x,
				cell_count_z: size.z,
				cell_distance: TCell::CELL_DISTANCE,
			};
			let min = context.grid_min();

			for ((x, z), cell) in cells.iter() {
				match cell {
					Agent::Player => {
						commands.spawn((
							Player,
							AgentOfPersistentMap(*persistent_entity),
							transform::<TCell>(x, z, min),
						));
					}
					Agent::Enemy => {
						commands.spawn((
							Enemy,
							AgentOfPersistentMap(*persistent_entity),
							transform::<TCell>(x, z, min),
						));
					}
					_ => {}
				}
			}
			commands.try_insert_on(entity, AgentsLoaded);
		}
	}
}

fn transform<TCell>(x: &u32, z: &u32, min: Vec3) -> Transform
where
	TCell: GridCellDistanceDefinition,
{
	let translation = min + Vec3::new(*x as f32, 0., *z as f32) * *TCell::CELL_DISTANCE;
	Transform::from_translation(translation)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		cell_grid_size::CellGridSize,
		grid_graph::grid_context::{CellCount, CellDistance},
	};
	use macros::new_valid;
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_count, assert_eq_unordered};

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell;

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: CellDistance = new_valid!(CellDistance, 4.);
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, MapCells::<Agent<_Cell>>::spawn_map_agents);

		app
	}

	macro_rules! entities_with {
		($ty:ty, $app:expr) => {
			$app.world().iter_entities().filter(|e| e.contains::<$ty>())
		};
	}

	#[test]
	fn spawn_player_on_1_by_1_grid() {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			size: CellGridSize {
				x: new_valid!(CellCount, 1),
				z: new_valid!(CellCount, 1),
			},
			cells: HashMap::from([((0, 0), Agent::<_Cell>::Player)]),
			..default()
		});

		app.update();

		let [player] = assert_count!(1, entities_with!(Player, app));
		assert_eq!(
			Some(&Transform::from_xyz(0., 0., 0.)),
			player.get::<Transform>(),
		);
	}

	#[test]
	fn spawn_enemy_on_1_by_1_grid() {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			size: CellGridSize {
				x: new_valid!(CellCount, 1),
				z: new_valid!(CellCount, 1),
			},
			cells: HashMap::from([((0, 0), Agent::<_Cell>::Enemy)]),
			..default()
		});

		app.update();

		let [enemy] = assert_count!(1, entities_with!(Enemy, app));
		assert_eq!(
			Some(&Transform::from_xyz(0., 0., 0.)),
			enemy.get::<Transform>(),
		);
	}

	#[test]
	fn spawn_on_3_by_3_grid() {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			size: CellGridSize {
				x: new_valid!(CellCount, 3),
				z: new_valid!(CellCount, 3),
			},
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

		app.update();

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
	}

	#[test]
	fn insert_agents_loaded() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				size: CellGridSize {
					x: new_valid!(CellCount, 1),
					z: new_valid!(CellCount, 1),
				},
				cells: HashMap::from([((0, 0), Agent::<_Cell>::Player)]),
				..default()
			})
			.id();

		app.update();

		assert!(app.world().entity(entity).contains::<AgentsLoaded>());
	}

	#[test]
	fn spawn_player_with_reference() {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		app.world_mut().spawn((
			persistent_entity,
			MapCells {
				size: CellGridSize {
					x: new_valid!(CellCount, 1),
					z: new_valid!(CellCount, 1),
				},
				cells: HashMap::from([((0, 0), Agent::<_Cell>::Player)]),
				..default()
			},
		));

		app.update();

		let [player] = assert_count!(1, entities_with!(Player, app));
		assert_eq!(
			Some(&AgentOfPersistentMap(persistent_entity)),
			player.get::<AgentOfPersistentMap>(),
		);
	}

	#[test]
	fn spawn_enemy_with_reference() {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		app.world_mut().spawn((
			persistent_entity,
			MapCells {
				size: CellGridSize {
					x: new_valid!(CellCount, 1),
					z: new_valid!(CellCount, 1),
				},
				cells: HashMap::from([((0, 0), Agent::<_Cell>::Enemy)]),
				..default()
			},
		));

		app.update();

		let [enemy] = assert_count!(1, entities_with!(Enemy, app));
		assert_eq!(
			Some(&AgentOfPersistentMap(persistent_entity)),
			enemy.get::<AgentOfPersistentMap>(),
		);
	}
}
