use crate::traits::try_insert_on::TryInsertOn;
use bevy::prelude::*;

impl<TComponent> AddTo for TComponent where TComponent: Component + Default {}

pub trait AddTo
where
	Self: Component + Default + Sized,
{
	fn add_to<TTarget>(commands: Commands, targets: Query<Entity, (With<TTarget>, Without<Self>)>)
	where
		TTarget: Component,
	{
		add_component_to(commands, targets);
	}
}

fn add_component_to<TCommands, TComponent, TTarget>(
	mut commands: TCommands,
	targets: Query<Entity, (With<TTarget>, Without<TComponent>)>,
) where
	TCommands: TryInsertOn,
	TComponent: Component + Default,
	TTarget: Component,
{
	for target in &targets {
		commands.try_insert_on(target, TComponent::default());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{simple_init, traits::mock::Mock};
	use bevy::ecs::system::RunSystemOnce;
	use mockall::{mock, predicate::eq};

	#[derive(Component, Debug, PartialEq)]
	struct _Target;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component;

	mock! {
		_Commands {}
		impl TryInsertOn for _Commands {
			fn try_insert_on<TBundle: Bundle>(&mut self, entity: Entity, bundle: TBundle) ;
		}
	}

	simple_init!(Mock_Commands);

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn add_to_target() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Target).id();

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_Component>()
				.with(eq(agent), eq(_Component))
				.times(1)
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			add_component_to::<In<Mock_Commands>, _Component, _Target>,
		);
	}

	#[test]
	fn do_not_add_to_target_if_already_added() {
		let mut app = setup();
		app.world_mut().spawn((_Target, _Component));

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_Component>()
				.never()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			add_component_to::<In<Mock_Commands>, _Component, _Target>,
		);
	}

	#[test]
	fn do_not_add_to_entity_without_target() {
		let mut app = setup();
		app.world_mut().spawn_empty();

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_Component>()
				.never()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			add_component_to::<In<Mock_Commands>, _Component, _Target>,
		);
	}
}
