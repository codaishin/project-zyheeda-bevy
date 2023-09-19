use crate::traits::set::Set;
use crate::Player;
use bevy::prelude::*;

pub fn schedule<
	TEvent: Copy + Event,
	TBehavior: From<TEvent>,
	TBehaviors: Set<TBehavior> + Component,
>(
	mut player: Query<(&mut TBehaviors, &Player)>,
	mut event_reader: EventReader<TEvent>,
) {
	let Ok((mut state, ..)) = player.get_single_mut() else {
		return; //FIXME: handle properly
	};

	for event in event_reader.iter() {
		state.set(TBehavior::from(*event));
	}
}

#[cfg(test)]
mod tests {
	use crate::components::UnitsPerSecond;

	use super::*;
	use bevy::prelude::App;
	use mockall::automock;

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
	struct _State {
		pub mock: Mock_State,
	}

	impl _State {
		fn new() -> Self {
			Self {
				mock: Mock_State::new(),
			}
		}
	}

	#[automock]
	impl Set<_Behavior> for _State {
		fn set(&mut self, value: _Behavior) {
			self.mock.set(value)
		}
	}

	#[test]
	fn do_schedule_move() {
		let mut app = App::new();
		let mut state = _State::new();
		let event = _Event {
			target: Vec3::new(1., 2., 3.),
		};

		app.add_systems(Update, schedule::<_Event, _Behavior, _State>);

		state
			.mock
			.expect_set()
			.withf(move |behavior| behavior.target == event.target)
			.times(1)
			.return_const(());
		app.world.spawn((
			Player {
				movement_speed: UnitsPerSecond::new(1.),
			},
			state,
		));
		app.add_event::<_Event>();
		app.world.resource_mut::<Events<_Event>>().send(event);
		app.update();
	}
}
