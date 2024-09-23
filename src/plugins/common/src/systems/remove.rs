use bevy::prelude::*;

pub fn remove_from<TAgent: Component, TComponent: Component>(
	mut commands: Commands,
	entities: Query<Entity, With<TAgent>>,
) {
	for entity in &entities {
		let Some(mut entity) = commands.get_entity(entity) else {
			continue;
		};

		entity.remove::<TComponent>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;

	#[derive(Component, Debug, PartialEq)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq)]
	struct _Component;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn remove_component() {
		let mut app = setup();
		let agent = app.world_mut().spawn((_Agent, _Component)).id();

		app.world_mut()
			.run_system_once(remove_from::<_Agent, _Component>);

		assert_eq!(None, app.world().entity(agent).get::<_Component>(),)
	}

	#[test]
	fn do_not_remove_component_when_no_agent() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Component).id();

		app.world_mut()
			.run_system_once(remove_from::<_Agent, _Component>);

		assert_eq!(
			Some(&_Component),
			app.world().entity(agent).get::<_Component>(),
		)
	}
}
