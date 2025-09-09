use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl<T> InsertProcessComponent for T where T: Component + Sized {}

pub trait InsertProcessComponent: Component + Sized {
	fn insert_process_component<TInput>(
		In(input): In<ProcessInput<TInput>>,
		mut commands: ZyheedaCommands,
		players: Query<Entity, With<Self>>,
	) where
		TInput: InputProcessComponent,
		TInput::TInputProcessComponent: From<TInput> + StopMovement,
	{
		let Ok(player) = players.single() else {
			return;
		};

		let component = match input {
			ProcessInput::New(input) => TInput::TInputProcessComponent::from(input),
			ProcessInput::Stop => TInput::TInputProcessComponent::stop(),
			ProcessInput::None => return,
		};

		commands.try_apply_on(&player, |mut e| {
			e.try_insert(component);
		});
	}
}

pub(crate) trait InputProcessComponent: Sized {
	type TInputProcessComponent: Component;
}

pub(crate) trait StopMovement {
	fn stop() -> Self;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ProcessInput<TInput> {
	New(TInput),
	Stop,
	None,
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Clone, Copy)]
	struct _Input(Vec3);

	impl InputProcessComponent for _Input {
		type TInputProcessComponent = _Movement;
	}

	#[derive(Component, Debug, PartialEq)]
	enum _Movement {
		To(Vec3),
		Stop,
	}

	impl StopMovement for _Movement {
		fn stop() -> Self {
			Self::Stop
		}
	}

	impl From<_Input> for _Movement {
		fn from(_Input(target): _Input) -> Self {
			Self::To(target)
		}
	}

	#[derive(Component)]
	struct _Player;

	fn setup(input: ProcessInput<_Input>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || input).pipe(_Player::insert_process_component),
		);

		app
	}

	#[test]
	fn trigger_movement() {
		let mut app = setup(ProcessInput::New(_Input(Vec3::new(1., 2., 3.))));
		let player = app.world_mut().spawn(_Player).id();

		app.update();

		assert_eq!(
			Some(&_Movement::To(Vec3::new(1., 2., 3.))),
			app.world().entity(player).get::<_Movement>(),
		);
	}

	#[test]
	fn default_movement() {
		let mut app = setup(ProcessInput::Stop);
		let player = app
			.world_mut()
			.spawn((_Player, _Movement::To(Vec3::new(1., 2., 3.))))
			.id();

		app.update();

		assert_eq!(
			Some(&_Movement::stop()),
			app.world().entity(player).get::<_Movement>(),
		);
	}

	#[test]
	fn do_nothing() {
		let mut app = setup(ProcessInput::None);
		let player = app
			.world_mut()
			.spawn((_Player, _Movement::To(Vec3::new(1., 2., 3.))))
			.id();

		app.update();

		assert_eq!(
			Some(&_Movement::To(Vec3::new(1., 2., 3.))),
			app.world().entity(player).get::<_Movement>(),
		);
	}
}
