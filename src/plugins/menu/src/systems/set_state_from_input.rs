use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
	state::state::FreelyMutableState,
};
use common::{
	states::game_state::GameState,
	tools::action_key::ActionKey,
	traits::{
		handles_input::{GetAllInputStates, InputState},
		states::PlayState,
	},
};

pub(crate) fn set_state_from_input<TState, TInput>(
	input: StaticSystemParam<TInput>,
	current_state: Res<State<GameState>>,
	next_state: ResMut<NextState<GameState>>,
) where
	GameState: From<TState>,
	TState: Copy + TryFrom<ActionKey> + 'static,
	for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
{
	internal_set_state_from_input::<GameState, TState, TInput>(input, current_state, next_state);
}

fn internal_set_state_from_input<TState, TStateVariant, TInput>(
	input: StaticSystemParam<TInput>,
	current_state: Res<State<TState>>,
	mut next_state: ResMut<NextState<TState>>,
) where
	TState: States + FreelyMutableState + PlayState + From<TStateVariant>,
	TStateVariant: Copy + TryFrom<ActionKey> + 'static,
	for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
{
	let current = current_state.get();
	let triggered_sub_states = input
		.get_all_input_states()
		.filter_map(just_pressed_action)
		.filter_map(sub_state::<TStateVariant>);

	for sub_state in triggered_sub_states {
		let target_state = match TState::from(sub_state) {
			state if &state == current => TState::play_state(),
			state => state,
		};
		next_state.set(target_state);
	}
}

fn just_pressed_action((a, i): (ActionKey, InputState)) -> Option<ActionKey> {
	match i {
		InputState::Pressed { just_now: true } => Some(a),
		_ => None,
	}
}

fn sub_state<T>(a: ActionKey) -> Option<T>
where
	T: TryFrom<ActionKey>,
{
	T::try_from(a).ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		state::app::{AppExtStates, StatesPlugin},
	};
	use common::{
		states::menu_state::MenuState,
		tools::is_not::IsNot,
		traits::iteration::IterFinite,
	};
	use macros::NestedMocks;
	use mockall::automock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Default, Debug, PartialEq, States, Hash, Eq, Clone, Copy)]
	enum _Wrapper {
		#[default]
		Default,
		Play,
		Wrap(_Variant),
	}

	impl PlayState for _Wrapper {
		fn play_state() -> Self {
			_Wrapper::Play
		}
	}

	#[derive(Debug, PartialEq, States, Hash, Eq, Clone, Copy)]
	struct _Variant;

	impl _Variant {
		const ACTION: ActionKey = ActionKey::Menu(MenuState::Settings);
	}

	impl TryFrom<ActionKey> for _Variant {
		type Error = IsNot<_Variant>;

		fn try_from(key: ActionKey) -> Result<Self, Self::Error> {
			match key {
				Self::ACTION => Ok(_Variant),
				_ => Err(IsNot::target_type()),
			}
		}
	}

	impl From<_Variant> for _Wrapper {
		fn from(variant: _Variant) -> Self {
			_Wrapper::Wrap(variant)
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetAllInputStates for _Input {
		fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
		where
			TAction: Into<ActionKey> + IterFinite + 'static,
		{
			self.mock.get_all_input_states()
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<_Wrapper>();
		app.insert_resource(input);
		app.add_systems(
			Update,
			internal_set_state_from_input::<_Wrapper, _Variant, Res<_Input>>,
		);

		app
	}

	#[test]
	fn set_state() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					_Variant::ACTION,
					InputState::just_pressed(),
				)))
			});
		}));

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_Wrapper>>().unwrap();
		assert_eq!(&_Wrapper::Wrap(_Variant), state.get());
	}

	#[test]
	fn do_not_set_state_when_not_just_pressed() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states()
				.returning(|| Box::new(std::iter::once((_Variant::ACTION, InputState::pressed()))));
		}));

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_Wrapper>>().unwrap();
		assert_eq!(&_Wrapper::Default, state.get());
	}

	#[test]
	fn set_to_play_if_variant_state_already_active() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					_Variant::ACTION,
					InputState::just_pressed(),
				)))
			});
		}));
		app.insert_resource(State::new(_Wrapper::Wrap(_Variant)));

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_Wrapper>>().unwrap();
		assert_eq!(&_Wrapper::Play, state.get());
	}
}
