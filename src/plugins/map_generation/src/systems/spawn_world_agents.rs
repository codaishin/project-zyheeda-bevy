use crate::{
	components::{
		map::{
			agents::AgentsLoaded,
			cells::{CellGrid, MapCells, agent::Agent},
		},
		map_agents::AgentOfPersistentMap,
		world_agent::WorldAgent,
	},
	grid_graph::grid_context::GridContext,
	traits::{GridCellDistanceDefinition, grid_min::GridMin},
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};

impl<TCell> MapCells<Agent<TCell>>
where
	TCell: GridCellDistanceDefinition + ThreadSafe,
{
	pub(crate) fn spawn_world_agents(
		mut commands: ZyheedaCommands,
		cells: Query<(Entity, &PersistentEntity, &Self)>,
	) {
		for (entity, persistent_entity, MapCells { definition, .. }) in &cells {
			let context = GridContext {
				cell_count_x: definition.size.x,
				cell_count_z: definition.size.z,
				cell_distance: TCell::CELL_DISTANCE,
			};
			let min = context.grid_min();
			let CellGrid(cells) = &definition.cells;

			for ((x, z), cell) in cells.iter() {
				let Agent::Some(agent_type) = cell else {
					continue;
				};

				commands.spawn((
					WorldAgent(*agent_type),
					AgentOfPersistentMap(*persistent_entity),
					transform::<TCell>(x, z, min),
				));
			}
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(AgentsLoaded);
			});
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
		traits::map_cells_extra::CellGridDefinition,
	};
	use common::traits::{handles_enemies::EnemyType, handles_map_generation::AgentType};
	use macros::new_valid;
	use testing::{SingleThreadedApp, assert_count, assert_eq_unordered};

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell;

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: CellDistance = new_valid!(CellDistance, 4.);
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, MapCells::<Agent<_Cell>>::spawn_world_agents);

		app
	}

	#[test]
	fn spawn_player_on_1_by_1_grid() {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			definition: CellGridDefinition {
				size: CellGridSize {
					x: new_valid!(CellCount, 1),
					z: new_valid!(CellCount, 1),
				},
				cells: CellGrid::from([((0, 0), Agent::<_Cell>::Some(AgentType::Player))]),
			},
			..default()
		});

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&Transform, With<WorldAgent>>();
		let [agent] = assert_count!(1, agents.iter(app.world()));
		assert_eq!(&Transform::from_xyz(0., 0., 0.), agent);
	}

	#[test]
	fn spawn_enemy_on_1_by_1_grid() {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			definition: CellGridDefinition {
				size: CellGridSize {
					x: new_valid!(CellCount, 1),
					z: new_valid!(CellCount, 1),
				},
				cells: CellGrid::from([(
					(0, 0),
					Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
				)]),
			},
			..default()
		});

		app.update();

		let mut agents = app.world_mut().query::<(&Transform, &WorldAgent)>();
		let [(transform, agent)] = assert_count!(1, agents.iter(app.world()));
		assert_eq!(
			(
				&Transform::from_xyz(0., 0., 0.),
				&WorldAgent(AgentType::Enemy(EnemyType::VoidSphere)),
			),
			(transform, agent)
		);
	}

	#[test]
	fn spawn_on_3_by_3_grid() {
		let mut app = setup();
		app.world_mut().spawn(MapCells {
			definition: CellGridDefinition {
				size: CellGridSize {
					x: new_valid!(CellCount, 3),
					z: new_valid!(CellCount, 3),
				},
				cells: CellGrid::from([
					(
						(0, 0),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					),
					(
						(0, 1),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					),
					(
						(0, 2),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					),
					((1, 0), Agent::<_Cell>::Some(AgentType::Player)),
					(
						(1, 1),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					),
					(
						(1, 2),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					),
					(
						(2, 0),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					),
					(
						(2, 1),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					),
					(
						(2, 2),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					),
				]),
			},
			..default()
		});

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&Transform, With<WorldAgent>>();
		let agents = assert_count!(9, agents.iter(app.world()));
		assert_eq_unordered!(
			[
				&Transform::from_xyz(-4., 0., -4.),
				&Transform::from_xyz(-4., 0., 0.),
				&Transform::from_xyz(-4., 0., 4.),
				&Transform::from_xyz(0., 0., -4.),
				&Transform::from_xyz(0., 0., 0.),
				&Transform::from_xyz(0., 0., 4.),
				&Transform::from_xyz(4., 0., -4.),
				&Transform::from_xyz(4., 0., 0.),
				&Transform::from_xyz(4., 0., 4.),
			],
			agents
		);
	}

	#[test]
	fn insert_agents_loaded() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapCells {
				definition: CellGridDefinition {
					size: CellGridSize {
						x: new_valid!(CellCount, 1),
						z: new_valid!(CellCount, 1),
					},
					cells: CellGrid::from([((0, 0), Agent::<_Cell>::Some(AgentType::Player))]),
				},
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
				definition: CellGridDefinition {
					size: CellGridSize {
						x: new_valid!(CellCount, 1),
						z: new_valid!(CellCount, 1),
					},
					cells: CellGrid::from([((0, 0), Agent::<_Cell>::Some(AgentType::Player))]),
				},
				..default()
			},
		));

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&AgentOfPersistentMap, With<WorldAgent>>();
		let [agent] = assert_count!(1, agents.iter(app.world()));
		assert_eq!(&AgentOfPersistentMap(persistent_entity), agent);
	}

	#[test]
	fn spawn_enemy_with_reference() {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		app.world_mut().spawn((
			persistent_entity,
			MapCells {
				definition: CellGridDefinition {
					size: CellGridSize {
						x: new_valid!(CellCount, 1),
						z: new_valid!(CellCount, 1),
					},
					cells: CellGrid::from([(
						(0, 0),
						Agent::<_Cell>::Some(AgentType::Enemy(EnemyType::VoidSphere)),
					)]),
				},
				..default()
			},
		));

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&AgentOfPersistentMap, With<WorldAgent>>();
		let [agent] = assert_count!(1, agents.iter(app.world()));
		assert_eq!(&AgentOfPersistentMap(persistent_entity), agent);
	}
}
