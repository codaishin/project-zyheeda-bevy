use crate::components::{
	grid::Grid,
	map::objects::{MapObjectOf, MapObjects},
	map_agents::{GridAgent, GridAgentOf},
};
use bevy::prelude::*;
use common::prelude::*;

impl GridAgent {
	#[allow(clippy::type_complexity)]
	pub(crate) fn link_to_grid<TGraph>(
		mut commands: ZyheedaCommands,
		maps: Query<&MapObjects>,
		agents: Query<(Entity, &MapObjectOf), (With<Self>, Without<GridAgentOf>)>,
		grids: Query<Entity, With<Grid<TGraph>>>,
	) where
		TGraph: ThreadSafe,
	{
		for (entity, MapObjectOf(map)) in agents {
			let Ok(map_objects) = maps.get(*map) else {
				continue;
			};
			let Some(grid) = map_objects.iter().find(|obj| grids.contains(*obj)) else {
				continue;
			};
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(GridAgentOf(grid));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{IsChanged, SingleThreadedApp, assert_related_count};

	struct _Graph;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_systems(
			Update,
			(
				GridAgent::link_to_grid::<_Graph>,
				IsChanged::<GridAgentOf>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn add_link() {
		let mut app = setup();
		let map = app.world_mut().spawn_empty().id();
		let grid = app
			.world_mut()
			.spawn((MapObjectOf(map), Grid::<_Graph>::from(_Graph)))
			.id();
		let entity = app.world_mut().spawn((MapObjectOf(map), GridAgent)).id();

		app.update();

		let [agent] = assert_related_count!(GridAgentOf, 1, app, grid);
		assert_eq!(entity, agent.id());
	}

	#[test]
	fn do_not_work_on_non_grid_agents() {
		let mut app = setup();
		let map = app.world_mut().spawn_empty().id();
		let grid = app
			.world_mut()
			.spawn((MapObjectOf(map), Grid::<_Graph>::from(_Graph)))
			.id();
		app.world_mut().spawn(MapObjectOf(map));

		app.update();

		assert_related_count!(GridAgentOf, 0, app, grid);
	}

	#[test]
	fn do_not_use_wrong_grid() {
		struct _OtherGraph;

		let mut app = setup();
		let map = app.world_mut().spawn_empty().id();
		let grid = app
			.world_mut()
			.spawn((MapObjectOf(map), Grid::<_OtherGraph>::from(_OtherGraph)))
			.id();
		app.world_mut().spawn((MapObjectOf(map), GridAgent));

		app.update();

		assert_related_count!(GridAgentOf, 0, app, grid);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let map = app.world_mut().spawn_empty().id();
		let grid = app
			.world_mut()
			.spawn((MapObjectOf(map), Grid::<_Graph>::from(_Graph)))
			.id();
		app.world_mut().spawn((MapObjectOf(map), GridAgent));

		app.update();
		app.update();

		let [agent] = assert_related_count!(GridAgentOf, 1, app, grid);
		assert_eq!(
			Some(&IsChanged::FALSE),
			agent.get::<IsChanged<GridAgentOf>>(),
		);
	}
}
