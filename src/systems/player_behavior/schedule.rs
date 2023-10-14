use crate::events::Enqueue;
use crate::traits::{add::Add, set::Set};
use crate::Player;
use bevy::prelude::*;

pub fn schedule<
	TEvent: Copy + Event,
	TBehavior: From<TEvent>,
	TBehaviors: Set<TBehavior> + Add<TBehavior> + Component,
>(
	mut player: Query<(&mut TBehaviors, &Player)>,
	mut event_reader: EventReader<TEvent>,
	mut enqueue_event_reader: EventReader<Enqueue<TEvent>>,
) {
	let Ok((mut behaviors, ..)) = player.get_single_mut() else {
		return; //FIXME: handle properly
	};

	for event in enqueue_event_reader.iter() {
		behaviors.add(TBehavior::from(event.0));
	}

	for event in event_reader.iter() {
		behaviors.set(TBehavior::from(*event));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::App;
	use mockall::mock;

	#[derive(Event, Clone, Copy)]
	struct _Event {
		pub target: Vec3,
	}

	pub struct _Behavior {
		pub target: Vec3,
	}

	impl From<_Event> for _Behavior {
		fn from(event: _Event) -> Self {
			Self {
				target: event.target,
			}
		}
	}

	#[derive(Component)]
	struct _Behaviors {
		pub mock: Mock_Behaviors,
	}

	impl _Behaviors {
		fn new() -> Self {
			Self {
				mock: Mock_Behaviors::new(),
			}
		}
	}

	impl Set<_Behavior> for _Behaviors {
		fn set(&mut self, value: _Behavior) {
			self.mock.set(value)
		}
	}

	impl Add<_Behavior> for _Behaviors {
		fn add(&mut self, value: _Behavior) {
			self.mock.add(value)
		}
	}

	mock! {
		pub _Behaviors {}
		impl Set<_Behavior> for _Behaviors {
			fn set(&mut self, value: _Behavior);
		}
		impl Add<_Behavior> for _Behaviors {
			fn add(&mut self, value: _Behavior);
		}
	}

	fn setup_app() -> App {
		let mut app = App::new();
		app.add_event::<_Event>();
		app.add_event::<Enqueue<_Event>>();

		app
	}

	#[test]
	fn set_movement() {
		let mut app = setup_app();
		let mut behaviors = _Behaviors::new();
		let event = _Event {
			target: Vec3::new(1., 2., 3.),
		};

		app.add_systems(Update, schedule::<_Event, _Behavior, _Behaviors>);

		behaviors
			.mock
			.expect_set()
			.withf(move |behavior| behavior.target == event.target)
			.times(1)
			.return_const(());
		app.world.spawn((Player { ..default() }, behaviors));

		app.world.resource_mut::<Events<_Event>>().send(event);
		app.update();
	}

	#[test]
	fn add_movement() {
		let mut app = setup_app();
		let mut behaviors = _Behaviors::new();
		let event = Enqueue(_Event {
			target: Vec3::new(1., 2., 3.),
		});

		app.add_systems(Update, schedule::<_Event, _Behavior, _Behaviors>);

		behaviors
			.mock
			.expect_add()
			.withf(move |behavior| behavior.target == event.0.target)
			.times(1)
			.return_const(());
		app.world.spawn((Player { ..default() }, behaviors));

		app.world
			.resource_mut::<Events<Enqueue<_Event>>>()
			.send(event);
		app.update();
	}
}
