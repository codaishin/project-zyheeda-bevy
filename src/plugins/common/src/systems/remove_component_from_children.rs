use crate::{
	components::Unmovable,
	tools::apply_recursively,
	traits::try_remove_from::TryRemoveFrom,
};
use bevy::prelude::*;

pub trait RemoveFromChildren<TComponent: Component> {
	fn remove_from_children_of<TAgent>(
		commands: Commands,
		agents: Query<Entity, With<TAgent>>,
		children: Query<&Children>,
		components_lookup: Query<(), (With<TComponent>, Without<Unmovable<TComponent>>)>,
		agents_lookup: Query<(), With<TAgent>>,
	) where
		TAgent: Component,
	{
		remove_from_children_of(commands, agents, children, components_lookup, agents_lookup);
	}
}

impl<TComponent: Component> RemoveFromChildren<TComponent> for TComponent {}

fn remove_from_children_of<TCommands, TAgent, TComponent>(
	mut commands: TCommands,
	agents: Query<Entity, With<TAgent>>,
	children: Query<&Children>,
	components_lookup: Query<(), (With<TComponent>, Without<Unmovable<TComponent>>)>,
	agents_lookup: Query<(), With<TAgent>>,
) where
	TCommands: TryRemoveFrom,
	TAgent: Component,
	TComponent: Component,
{
	if agents.is_empty() {
		return;
	}

	let get_children = &|entity| children.get(entity).ok().map(|c| c.iter());
	let has_component = &|entity| components_lookup.contains(entity);
	let is_no_agent = &|entity| !agents_lookup.contains(entity);
	let remove_component = &mut |entity| commands.try_remove_from::<TComponent>(entity);

	for entity in &agents {
		apply_recursively(
			entity,
			remove_component,
			get_children,
			has_component,
			is_no_agent,
		);
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use crate::{components::Unmovable, simple_init, traits::mock::Mock};
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
	fn do_not_remove_when_unmovable() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((_Component, Unmovable::<_Component>::default()))
			.set_parent(parent);

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

	#[test]
	fn do_not_remove_from_deep_child_multiple_times_when_nested() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Agent).id();
		let child = app.world_mut().spawn(_Agent).set_parent(parent).id();
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
