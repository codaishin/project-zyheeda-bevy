use crate::components::map::{agents::AgentsLoaded, cells::agent::Agent, folder::MapFolder};
use bevy::prelude::*;
use common::traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn};

type WithoutAgentsAndNew<T> = (Added<T>, Without<AgentsLoaded>);

impl<TCell> MapFolder<TCell>
where
	TCell: ThreadSafe,
{
	pub(crate) fn load_agents(
		mut commands: Commands,
		folders: Query<(Entity, &Self), WithoutAgentsAndNew<Self>>,
	) {
		for (entity, MapFolder { path, .. }) in &folders {
			commands.try_insert_on(entity, MapFolder::<Agent<TCell>>::from(path.clone()));
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::components::map::agents::AgentsLoaded;

	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Cell;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, MapFolder::<_Cell>::load_agents);

		app
	}

	#[test]
	fn insert_agents_folder() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapFolder::<_Cell>::from("my/path"))
			.id();

		app.update();

		assert_eq!(
			Some(&MapFolder::<Agent<_Cell>>::from("my/path")),
			app.world()
				.entity(entity)
				.get::<MapFolder::<Agent<_Cell>>>(),
		);
	}

	#[test]
	fn do_not_insert_agents_folder_when_agents_loaded() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((MapFolder::<_Cell>::from("my/path"), AgentsLoaded))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<MapFolder::<Agent<_Cell>>>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MapFolder::<_Cell>::from("my/path"))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<MapFolder<Agent<_Cell>>>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<MapFolder::<Agent<_Cell>>>(),
		);
	}
}
