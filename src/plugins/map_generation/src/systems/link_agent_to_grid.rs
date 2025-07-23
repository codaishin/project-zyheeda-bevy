use crate::components::{
	grid::Grid,
	map_agents::{AgentOfPersistentMap, GridAgentOf},
	nav_grid::NavGrid,
};
use bevy::prelude::*;
use common::{
	resources::persistent_entities::PersistentEntities,
	traits::try_insert_on::TryInsertOn,
};

impl AgentOfPersistentMap {
	pub(crate) fn link_to_grid(
		mut persistent_entities: ResMut<PersistentEntities>,
		mut commands: Commands,
		maps: Query<&NavGrid<Grid>>,
		agents: Query<(Entity, &Self), Changed<Self>>,
	) {
		for (entity, AgentOfPersistentMap(map)) in &agents {
			let Some(map) = persistent_entities.get_entity(map) else {
				continue;
			};
			let Ok(nav_grid) = maps.get(map) else {
				continue;
			};
			commands.try_insert_on(entity, GridAgentOf(nav_grid.entity));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::map_agents::GridAgentOf;
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_systems(Update, AgentOfPersistentMap::link_to_grid);

		app
	}

	#[test]
	fn add_link() {
		let mut app = setup();
		let grid = app.world_mut().spawn_empty().id();
		let map = PersistentEntity::default();
		app.world_mut().spawn((map, NavGrid::<Grid>::from(grid)));
		let entity = app.world_mut().spawn(AgentOfPersistentMap(map)).id();

		app.update();

		assert_eq!(
			Some(&GridAgentOf(grid)),
			app.world().entity(entity).get::<GridAgentOf>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let grid = app.world_mut().spawn_empty().id();
		let map = PersistentEntity::default();
		app.world_mut().spawn((map, NavGrid::<Grid>::from(grid)));
		let entity = app.world_mut().spawn(AgentOfPersistentMap(map)).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<GridAgentOf>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<GridAgentOf>());
	}

	#[test]
	fn act_again_when_new_map_reference_inserted() {
		let mut app = setup();
		let grid = app.world_mut().spawn_empty().id();
		let map = PersistentEntity::default();
		app.world_mut().spawn((map, NavGrid::<Grid>::from(grid)));
		let entity = app.world_mut().spawn(AgentOfPersistentMap(map)).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<GridAgentOf>()
			.insert(AgentOfPersistentMap(map));
		app.update();

		assert_eq!(
			Some(&GridAgentOf(grid)),
			app.world().entity(entity).get::<GridAgentOf>(),
		);
	}
}
