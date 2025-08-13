use crate::{
	components::protected::Protected,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;

impl<TComponent: Component> Remove<TComponent> for TComponent {}

pub trait Remove<TAgent>
where
	TAgent: Component,
{
	fn remove_from_self_and_children<TComponent>(
		mut commands: ZyheedaCommands,
		agents: Query<(), With<TAgent>>,
		entities: Query<Entity, (With<TComponent>, Without<Protected<TComponent>>)>,
		parents: Query<&ChildOf>,
	) where
		TComponent: Component,
	{
		let has_agent = |entity| agents.contains(entity);
		let parent_or_self_has_agent =
			|entity: &Entity| has_agent(*entity) || parents.iter_ancestors(*entity).any(has_agent);

		for entity in entities.iter().filter(parent_or_self_has_agent) {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<TComponent>();
			});
		}
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Agent::remove_from_self_and_children::<_Component>);

		app
	}

	#[test]
	fn remove_from_agent() {
		let mut app = setup();
		let parent = app.world_mut().spawn((_Agent, _Component)).id();

		app.update();

		assert_eq!(None, app.world().entity(parent).get::<_Component>());
	}

	#[test]
	fn remove_from_child() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app
			.world_mut()
			.spawn(_Component)
			.insert(ChildOf(parent))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(child).get::<_Component>());
	}

	#[test]
	fn do_not_remove_when_parent_no_agent() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Component).id();

		app.update();

		assert_eq!(
			Some(&_Component),
			app.world().entity(parent).get::<_Component>(),
		);
	}

	#[test]
	fn do_not_remove_when_protected() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app
			.world_mut()
			.spawn((_Component, Protected::<_Component>::default()))
			.insert(ChildOf(parent))
			.id();

		app.update();

		assert_eq!(
			Some(&_Component),
			app.world().entity(child).get::<_Component>(),
		);
	}

	#[test]
	fn remove_from_deep_child() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app.world_mut().spawn_empty().insert(ChildOf(parent)).id();
		let deep_child = app
			.world_mut()
			.spawn(_Component)
			.insert(ChildOf(child))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(deep_child).get::<_Component>(),);
	}
}
