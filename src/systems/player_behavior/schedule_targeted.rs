use crate::traits::{add::Add, new::New1, target::Target};
use crate::Player;
use bevy::prelude::*;

pub fn schedule_targeted<
	TBehavior: New1<Vec3>,
	TEvent: Target + Event,
	TState: Add<TBehavior> + Component,
>(
	mut player: Query<(&mut TState, &Player)>,
	mut event_reader: EventReader<TEvent>,
) {
	let Ok((mut state, ..)) = player.get_single_mut() else {
		return; //FIXME: handle properly
	};

	for event in event_reader.iter() {
		state.add(TBehavior::new(event.target()));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::App;
	use mockall::automock;

	#[derive(Event)]
	struct _Event {
		pub target: Vec3,
	}

	impl Target for _Event {
		fn target(&self) -> Vec3 {
			self.target
		}
	}

	pub struct _Behavior {
		pub target: Vec3,
	}

	impl New1<Vec3> for _Behavior {
		fn new(target: Vec3) -> Self {
			Self { target }
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
	impl Add<_Behavior> for _State {
		fn add(&mut self, value: _Behavior) {
			self.mock.add(value)
		}
	}

	#[test]
	fn do_schedule_move() {
		let mut app = App::new();
		let mut state = _State::new();
		let event = _Event {
			target: Vec3::new(1., 2., 3.),
		};

		app.add_systems(Update, schedule_targeted::<_Behavior, _Event, _State>);

		state
			.mock
			.expect_add()
			.withf(move |behavior| behavior.target == event.target)
			.times(1)
			.return_const(());
		app.world.spawn((Player, state));
		app.add_event::<_Event>();
		app.world.resource_mut::<Events<_Event>>().send(event);
		app.update();
	}
}
