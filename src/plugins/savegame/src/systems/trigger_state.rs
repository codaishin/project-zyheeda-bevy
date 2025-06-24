use bevy::{prelude::*, state::state::FreelyMutableState};
use common::{tools::action_key::user_input::UserInput, traits::key_mappings::JustPressed};

impl<T, TActionKey> TriggerState<TActionKey> for T
where
	T: Resource + JustPressed<TActionKey>,
	TActionKey: PartialEq + Copy,
{
}

pub(crate) trait TriggerState<TActionKey>: Resource + JustPressed<TActionKey>
where
	TActionKey: PartialEq + Copy,
{
	fn trigger<TState>(
		action: TActionKey,
		state: TState,
	) -> impl Fn(Res<Self>, Res<ButtonInput<UserInput>>, ResMut<NextState<TState>>)
	where
		TState: FreelyMutableState + Copy,
	{
		move |key_map, input, mut game_state| {
			if !key_map.just_pressed(&input).any(is(action)) {
				return;
			}

			game_state.set(state);
		}
	}
}

fn is<TActionKey>(action: TActionKey) -> impl Fn(TActionKey) -> bool
where
	TActionKey: PartialEq,
{
	move |pressed| pressed == action
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::state::app::StatesPlugin;
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::automock;

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

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl JustPressed<_Action> for _Map {
		fn just_pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = _Action> {
			self.mock.just_pressed(input)
		}
	}

	fn setup(map: _Map, input: ButtonInput<UserInput>, action: _Action, state: _State) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<_State>();
		app.insert_resource(input);
		app.insert_resource(map);
		app.add_systems(Update, _Map::trigger(action, state));

		app
	}

	#[test]
	fn trigger_a() {
		let map = _Map::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::once(_Action::A)));
		});
		let mut app = setup(map, ButtonInput::default(), _Action::A, _State::A);

		app.update();

		assert!(matches!(
			app.world().get_resource::<NextState<_State>>(),
			Some(NextState::Pending(_State::A))
		));
	}

	#[test]
	fn trigger_b() {
		let map = _Map::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::once(_Action::B)));
		});
		let mut app = setup(map, ButtonInput::default(), _Action::B, _State::B);

		app.update();

		assert!(matches!(
			app.world().get_resource::<NextState<_State>>(),
			Some(NextState::Pending(_State::B))
		));
	}

	#[test]
	fn call_key_map_with_correct_arguments() {
		let map = _Map::new().with_mock(|mock| {
			mock.expect_just_pressed().times(1).returning(|input| {
				assert!(input.just_pressed(UserInput::MouseButton(MouseButton::Forward)));
				Box::new(std::iter::empty())
			});
		});
		let mut input = ButtonInput::default();
		input.press(UserInput::MouseButton(MouseButton::Forward));
		let mut app = setup(map, input, _Action::A, _State::A);

		app.update();
	}
}
