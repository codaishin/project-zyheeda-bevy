use crate::{
	components::protected::Protected,
	tools::get_recursively::{get_recursively_from, related::Child},
	traits::try_remove_from::TryRemoveFrom,
};
use bevy::prelude::*;

pub trait Remove<TAgent>
where
	TAgent: Component,
{
	fn remove_from_self_and_children<TComponent>(
		commands: Commands,
		agents: Query<Entity, With<TAgent>>,
		children: Query<&Children>,
		components_lookup: Query<(), (With<TComponent>, Without<Protected<TComponent>>)>,
		agents_lookup: Query<(), With<TAgent>>,
	) where
		TComponent: Component,
	{
		remove_components(commands, agents, children, components_lookup, agents_lookup);
	}
}

impl<TComponent: Component> Remove<TComponent> for TComponent {}

fn remove_components<TCommands, TAgent, TComponent>(
	mut commands: TCommands,
	agents: Query<Entity, With<TAgent>>,
	children: Query<&Children>,
	components_lookup: Query<(), (With<TComponent>, Without<Protected<TComponent>>)>,
	agents_lookup: Query<(), With<TAgent>>,
) where
	TCommands: TryRemoveFrom,
	TAgent: Component,
	TComponent: Component,
{
	if agents.is_empty() {
		return;
	}

	let children = &|entity| children.get(entity).ok().map(|c| c.iter().map(Child::new));
	let has_component = &|entity: &Entity| components_lookup.contains(*entity);
	let is_no_agent = &|Child(entity): &Child| !agents_lookup.contains(*entity);

	for entity in &agents {
		for entity in get_recursively_from(entity, children, is_no_agent).filter(has_component) {
			commands.try_remove_from::<TComponent>(entity)
		}
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use mockall::{mock, predicate::eq};
	use testing::{Mock, simple_init};

	#[derive(Component, Debug, PartialEq)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq)]
	struct _Component;

	mock! {
		_Commands {}
		impl TryRemoveFrom for _Commands {
			fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity);
		}
	}

	simple_init!(Mock_Commands);

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn remove_from_agent() -> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app.world_mut().spawn((_Agent, _Component)).id();

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.times(1)
				.with(eq(parent))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			remove_components::<In<Mock_Commands>, _Agent, _Component>,
			mock,
		)
	}

	#[test]
	fn remove_from_child() -> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app
			.world_mut()
			.spawn(_Component)
			.insert(ChildOf(parent))
			.id();

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.times(1)
				.with(eq(child))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			remove_components::<In<Mock_Commands>, _Agent, _Component>,
			mock,
		)
	}

	#[test]
	fn do_not_remove_when_parent_no_agent() -> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		app.world_mut().spawn(_Component).insert(ChildOf(parent));

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.never()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			remove_components::<In<Mock_Commands>, _Agent, _Component>,
			mock,
		)
	}

	#[test]
	fn do_not_remove_when_protected() -> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((_Component, Protected::<_Component>::default()))
			.insert(ChildOf(parent));

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.never()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			remove_components::<In<Mock_Commands>, _Agent, _Component>,
			mock,
		)
	}

	#[test]
	fn remove_from_deep_child() -> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app.world_mut().spawn_empty().insert(ChildOf(parent)).id();
		let deep_child = app
			.world_mut()
			.spawn(_Component)
			.insert(ChildOf(child))
			.id();

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.times(1)
				.with(eq(deep_child))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			remove_components::<In<Mock_Commands>, _Agent, _Component>,
			mock,
		)
	}

	#[test]
	fn do_not_remove_from_deep_child_multiple_times_when_nested() -> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app.world_mut().spawn(_Agent).insert(ChildOf(parent)).id();
		let deep_child = app
			.world_mut()
			.spawn(_Component)
			.insert(ChildOf(child))
			.id();

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.times(1)
				.with(eq(deep_child))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			remove_components::<In<Mock_Commands>, _Agent, _Component>,
			mock,
		)
	}
}
