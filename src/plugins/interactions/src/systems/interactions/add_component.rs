use bevy::prelude::{Added, Commands, Component, Entity, Query};
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn add_component_to<TAgent: Component, TComponent: Component + Default>(
	mut commands: Commands,
	agents: Query<(Entity, Option<&TComponent>), Added<TAgent>>,
) {
	for (entity, interactions) in &agents {
		if interactions.is_some() {
			continue;
		}
		commands.try_insert_on(entity, TComponent::default());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component(&'static str);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, add_component_to::<_Agent, _Component>);

		app
	}

	#[test]
	fn insert_interacting_entities() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();

		assert_eq!(
			Some(&_Component("")),
			app.world().entity(agent).get::<_Component>()
		)
	}

	#[test]
	fn do_not_insert_interacting_entities_when_no_agent_present() {
		let mut app = setup();
		let agent = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Component>())
	}

	#[test]
	fn do_not_insert_interacting_entities_when_already_present() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((_Agent, _Component("already present")))
			.id();

		app.update();

		assert_eq!(
			Some(&_Component("already present")),
			app.world().entity(agent).get::<_Component>()
		)
	}

	#[test]
	fn do_not_insert_interacting_entities_when_agent_not_new() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();
		app.world_mut().entity_mut(agent).remove::<_Component>();
		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Component>())
	}
}
