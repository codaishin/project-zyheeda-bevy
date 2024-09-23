use crate::traits::try_remove_from::TryRemoveFrom;
use bevy::prelude::*;

type Agents<'w, 's, TAgent, TComponent> = Query<'w, 's, Entity, (With<TAgent>, With<TComponent>)>;

pub fn remove_from<TAgent, TComponent>(commands: Commands, agents: Agents<TAgent, TComponent>)
where
	TAgent: Component,
	TComponent: Component,
{
	inner_remove_from::<Commands, TAgent, TComponent>(commands, agents);
}

fn inner_remove_from<TCommands, TAgent, TComponent>(
	mut commands: TCommands,
	agents: Agents<TAgent, TComponent>,
) where
	TCommands: TryRemoveFrom,
	TAgent: Component,
	TComponent: Component,
{
	for entity in &agents {
		commands.try_remove_from::<TComponent>(entity);
	}
}

#[cfg(test)]
mod tests {
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
	fn remove_component() {
		let mut app = setup();
		let entity = app.world_mut().spawn((_Agent, _Component)).id();

		let mock = Mock_Commands::new_mock(move |mock| {
			mock.expect_try_remove_from::<_Component>()
				.times(1)
				.with(eq(entity))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			mock,
			inner_remove_from::<In<Mock_Commands>, _Agent, _Component>,
		);
	}

	#[test]
	fn do_not_remove_component_when_no_agent() {
		let mut app = setup();
		app.world_mut().spawn(_Component);

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.never()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			mock,
			inner_remove_from::<In<Mock_Commands>, _Agent, _Component>,
		);
	}

	#[test]
	fn do_not_try_to_remove_component_when_no_component() {
		let mut app = setup();
		app.world_mut().spawn(_Agent);

		let mock = Mock_Commands::new_mock(|mock| {
			mock.expect_try_remove_from::<_Component>()
				.never()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			mock,
			inner_remove_from::<In<Mock_Commands>, _Agent, _Component>,
		);
	}
}
