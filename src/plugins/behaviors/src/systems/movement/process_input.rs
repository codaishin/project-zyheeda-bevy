use bevy::prelude::*;

impl<T> ProcessInput for T where T: Component + Sized {}

pub trait ProcessInput: Component + Sized {
	fn process<TInput>(
		mut commands: Commands,
		mut input: EventReader<TInput>,
		players: Query<Entity, With<Self>>,
	) where
		TInput: Event + EventProcessComponent,
		for<'a> TInput::TComponent: From<&'a TInput>,
	{
		let Ok(player) = players.single() else {
			return;
		};
		let Ok(mut player) = commands.get_entity(player) else {
			return;
		};

		for event in input.read() {
			player.try_insert(TInput::TComponent::from(event));
		}
	}
}

pub(crate) trait EventProcessComponent {
	type TComponent: Component;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Event)]
	struct _Event(Vec3);

	impl EventProcessComponent for _Event {
		type TComponent = _Movement;
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Movement(Vec3);

	impl From<&_Event> for _Movement {
		fn from(_Event(target): &_Event) -> Self {
			Self(*target)
		}
	}

	#[derive(Component)]
	struct _Player;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Player::process::<_Event>);
		app.add_event::<_Event>();

		app
	}

	#[test]
	fn trigger_movement() {
		let mut app = setup();
		let player = app.world_mut().spawn(_Player).id();
		app.world_mut().send_event(_Event(Vec3::new(1., 2., 3.)));

		app.update();

		assert_eq!(
			Some(&_Movement(Vec3::new(1., 2., 3.))),
			app.world().entity(player).get::<_Movement>(),
		);
	}
}
