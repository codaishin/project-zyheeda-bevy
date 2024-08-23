use crate::components::interacting_entities::InteractingEntities;
use bevy::prelude::{Added, Commands, Component, Entity, Query};
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn add_interacting_entities<TAgent: Component>(
	mut commands: Commands,
	agents: Query<(Entity, Option<&InteractingEntities>), Added<TAgent>>,
) {
	for (entity, interactions) in &agents {
		if interactions.is_some() {
			continue;
		}
		commands.try_insert_on(entity, InteractingEntities::default());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::interacting_entities::InteractingEntities;
	use bevy::app::{App, Update};
	use common::{components::ColliderRoot, test_tools::utils::SingleThreadedApp};

	#[derive(Component)]
	struct _Agent;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, add_interacting_entities::<_Agent>);

		app
	}

	#[test]
	fn insert_interacting_entities() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();

		assert_eq!(
			Some(&InteractingEntities::default()),
			app.world().entity(agent).get::<InteractingEntities>()
		)
	}

	#[test]
	fn do_not_insert_interacting_entities_when_no_agent_present() {
		let mut app = setup();
		let agent = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<InteractingEntities>())
	}

	#[test]
	fn do_not_insert_interacting_entities_when_already_present() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Agent,
				InteractingEntities::new([ColliderRoot(Entity::from_raw(100))]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&InteractingEntities::new([ColliderRoot(Entity::from_raw(
				100
			))])),
			app.world().entity(agent).get::<InteractingEntities>()
		)
	}

	#[test]
	fn do_not_insert_interacting_entities_when_agent_not_new() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.remove::<InteractingEntities>();
		app.update();

		assert_eq!(None, app.world().entity(agent).get::<InteractingEntities>())
	}
}
