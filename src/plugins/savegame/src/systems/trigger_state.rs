use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
	state::state::FreelyMutableState,
};
use common::{
	tools::action_key::ActionKey,
	traits::handles_input::{GetInputState, InputState},
};

impl<T> TriggerState for T where T: for<'w, 's> SystemParam<Item<'w, 's>: GetInputState> {}

pub(crate) trait TriggerState: for<'w, 's> SystemParam<Item<'w, 's>: GetInputState> {
	#[allow(clippy::type_complexity)]
	fn trigger<TActionKey, TState>(
		action: TActionKey,
		state: TState,
	) -> impl Fn(StaticSystemParam<Self>, ResMut<NextState<TState>>)
	where
		TActionKey: Into<ActionKey> + Copy + 'static,
		TState: FreelyMutableState + Copy,
	{
		move |input, mut game_state| {
			if input.get_input_state(action) != InputState::just_pressed() {
				return;
			}

			game_state.set(state);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::state::app::StatesPlugin;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
	enum _State {
		#[default]
		Default,
		A,
		B,
	}

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _Action {
		A,
		B,
	}

	impl From<_Action> for ActionKey {
		fn from(_: _Action) -> Self {
			panic!("NOT USED")
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetInputState for _Input {
		fn get_input_state<TAction>(&self, action: TAction) -> InputState
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.mock.get_input_state(action)
		}
	}

	fn setup(input: _Input, action: _Action, state: _State) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<_State>();
		app.insert_resource(input);
		app.add_systems(Update, Res::<_Input>::trigger(action, state));

		app
	}

	#[test]
	fn trigger_a() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(_Action::A))
				.return_const(InputState::just_pressed());
		});
		let mut app = setup(input, _Action::A, _State::A);

		app.update();

		assert!(matches!(
			app.world().get_resource::<NextState<_State>>(),
			Some(NextState::Pending(_State::A))
		));
	}

	#[test]
	fn trigger_b() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(_Action::B))
				.return_const(InputState::just_pressed());
		});
		let mut app = setup(input, _Action::B, _State::B);

		app.update();

		assert!(matches!(
			app.world().get_resource::<NextState<_State>>(),
			Some(NextState::Pending(_State::B))
		));
	}
}
