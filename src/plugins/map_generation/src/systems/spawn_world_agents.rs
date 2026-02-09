use crate::{
	components::{
		map::{
			agents::AgentsLoaded,
			cells::{CellGrid, MapCells, agent::Agent},
		},
		map_agents::AgentOfPersistentMap,
	},
	grid_graph::grid_context::GridContext,
	resources::agents::prefab::AgentPrefab,
	traits::{GridCellDistanceDefinition, grid_min::GridMin},
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

impl<TCell> MapCells<Agent<TCell>>
where
	TCell: GridCellDistanceDefinition + ThreadSafe,
{
	pub(crate) fn spawn_world_agents(
		mut commands: ZyheedaCommands,
		cells: Query<(Entity, &PersistentEntity, &Self)>,
		agent_prefab: Res<AgentPrefab>,
	) {
		let AgentPrefab(apply_prefab) = *agent_prefab;

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

				let entity = commands.spawn(AgentOfPersistentMap(*persistent_entity));
				apply_prefab(
					ZyheedaEntityCommands::from(entity),
					translation::<TCell>(x, z, min),
					*agent_type,
				);
			}
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(AgentsLoaded);
			});
		}
	}
}

fn translation<TCell>(x: &u32, z: &u32, min: Vec3) -> Vec3
where
	TCell: GridCellDistanceDefinition,
{
	min + Vec3::new(*x as f32, 0., *z as f32) * *TCell::CELL_DISTANCE
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		cell_grid_size::CellGridSize,
		grid_graph::grid_context::{CellCount, CellDistance},
		resources::agents::prefab::AgentPrefab,
		traits::map_cells_extra::CellGridDefinition,
	};
	use common::traits::{handles_enemies::EnemyType, handles_map_generation::AgentType};
	use macros::new_valid;
	use testing::{SingleThreadedApp, assert_count, assert_eq_unordered};

	#[derive(Component, Debug, PartialEq)]
	struct _NewAgent {
		ground_position: Vec3,
		agent_type: AgentType,
	}

	#[derive(Clone, Debug, PartialEq, TypePath)]
	struct _Cell;

	impl GridCellDistanceDefinition for _Cell {
		const CELL_DISTANCE: CellDistance = new_valid!(CellDistance, 4.);
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(AgentPrefab(|mut entity, ground_position, agent_type| {
			entity.try_insert(_NewAgent {
				ground_position,
				agent_type,
			});
		}));
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

		let mut agents = app.world_mut().query::<&_NewAgent>();
		let [agent] = assert_count!(1, agents.iter(app.world()));
		assert_eq!(
			&_NewAgent {
				agent_type: AgentType::Player,
				ground_position: Vec3::ZERO,
			},
			agent
		);
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

		let mut agents = app.world_mut().query::<&_NewAgent>();
		let [agent] = assert_count!(1, agents.iter(app.world()));
		assert_eq!(
			&_NewAgent {
				ground_position: Vec3::ZERO,
				agent_type: AgentType::Enemy(EnemyType::VoidSphere)
			},
			agent
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

		let mut agents = app.world_mut().query::<&_NewAgent>();
		let agents = assert_count!(9, agents.iter(app.world()));
		assert_eq_unordered!(
			[
				&_NewAgent {
					ground_position: Vec3::new(-4., 0., -4.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
				&_NewAgent {
					ground_position: Vec3::new(-4., 0., 4.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
				&_NewAgent {
					ground_position: Vec3::new(-4., 0., 0.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
				&_NewAgent {
					ground_position: Vec3::new(0., 0., -4.),
					agent_type: AgentType::Player,
				},
				&_NewAgent {
					ground_position: Vec3::new(0., 0., 0.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
				&_NewAgent {
					ground_position: Vec3::new(0., 0., 4.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
				&_NewAgent {
					ground_position: Vec3::new(4., 0., -4.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
				&_NewAgent {
					ground_position: Vec3::new(4., 0., 0.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
				&_NewAgent {
					ground_position: Vec3::new(4., 0., 4.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
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
			.query_filtered::<&AgentOfPersistentMap, With<_NewAgent>>();
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
			.query_filtered::<&AgentOfPersistentMap, With<_NewAgent>>();
		let [agent] = assert_count!(1, agents.iter(app.world()));
		assert_eq!(&AgentOfPersistentMap(persistent_entity), agent);
	}
}
