use bevy::prelude::*;

impl<T> InsertProcessComponent for T where T: Component + Sized {}

pub trait InsertProcessComponent: Component + Sized {
	fn insert_process_component<TInput>(
		In(input): In<Option<TInput>>,
		mut commands: Commands,
		players: Query<Entity, With<Self>>,
	) where
		TInput: InputProcessComponent,
	{
		let Some(input) = input else {
			return;
		};
		let Ok(player) = players.single() else {
			return;
		};
		let Ok(mut player) = commands.get_entity(player) else {
			return;
		};

		player.try_insert(TInput::TComponent::from(input));
	}
}

pub(crate) trait InputProcessComponent: Sized {
	type TComponent: Component + From<Self>;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Clone, Copy)]
	struct _Input(Vec3);

	impl InputProcessComponent for _Input {
		type TComponent = _Movement;
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Movement(Vec3);

	impl From<_Input> for _Movement {
		fn from(_Input(target): _Input) -> Self {
			Self(target)
		}
	}

	#[derive(Component)]
	struct _Player;

	fn setup(input: Option<_Input>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || input).pipe(_Player::insert_process_component),
		);

		app
	}

	#[test]
	fn trigger_movement() {
		let mut app = setup(Some(_Input(Vec3::new(1., 2., 3.))));
		let player = app.world_mut().spawn(_Player).id();

		app.update();

		assert_eq!(
			Some(&_Movement(Vec3::new(1., 2., 3.))),
			app.world().entity(player).get::<_Movement>(),
		);
	}
}
