use crate::traits::try_remove_from::TryRemoveFrom;
use bevy::prelude::*;

pub trait RemoveFromChildren<TComponent: Component> {
	fn remove_from_children_of<TAgent>(
		commands: Commands,
		agents: Query<Entity, With<TAgent>>,
		children: Query<&Children>,
		components: Query<(), With<TComponent>>,
	) where
		TAgent: Component,
	{
		remove_from_children_of(commands, agents, children, components);
	}
}

impl<TComponent: Component> RemoveFromChildren<TComponent> for TComponent {}

fn remove_from_children_of<TCommands, TAgent, TComponent>(
	mut commands: TCommands,
	agents: Query<Entity, With<TAgent>>,
	children: Query<&Children>,
	components: Query<(), With<TComponent>>,
) where
	TCommands: TryRemoveFrom,
	TAgent: Component,
	TComponent: Component,
{
	let children = |entity| children.iter_descendants(entity);
	let has_component = |entity| components.contains(entity);

	for child in agents.iter().flat_map(children) {
		if !has_component(child) {
			continue;
		}

		commands.try_remove_from::<TComponent>(child);
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use crate::{simple_init, traits::mock::Mock};
	use bevy::ecs::system::RunSystemOnce;
	use mockall::{mock, predicate::eq};

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

	impl TryRemoveFrom for In<Mock_Commands> {
		fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity) {
			self.0.try_remove_from::<TBundle>(entity);
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn remove_from_child() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app.world_mut().spawn(_Component).set_parent(parent).id();

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.times(1)
				.with(eq(child))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			mock,
			remove_from_children_of::<In<Mock_Commands>, _Agent, _Component>,
		);
	}

	#[test]
	fn do_not_remove_when_parent_no_agent() {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		app.world_mut().spawn(_Component).set_parent(parent);

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.never()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			mock,
			remove_from_children_of::<In<Mock_Commands>, _Agent, _Component>,
		);
	}

	#[test]
	fn remove_from_deep_child() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app.world_mut().spawn_empty().set_parent(parent).id();
		let deep_child = app.world_mut().spawn(_Component).set_parent(child).id();

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.times(1)
				.with(eq(deep_child))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			mock,
			remove_from_children_of::<In<Mock_Commands>, _Agent, _Component>,
		);
	}
}
