use crate::{resources::mouse_override::MouseOverride, system_params::input::Input};
use bevy::{ecs::system::SystemParam, input::ButtonInput, prelude::*};
use common::prelude::*;
use std::{hash::Hash, ops::Deref};
use zyheeda_core::prelude::*;

impl<'w, 's, TKeyMap> GetInputState for Input<'w, 's, TKeyMap>
where
	TKeyMap: SystemParam<Item<'w, 's>: GetInput> + 'static,
{
	fn get_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static,
	{
		let action_key = action.into();

		if let Some(input_state) = self.matching_mouse_override(action_key) {
			return input_state;
		}

		let input = self.key_map.get_input(action_key);

		if input == LEFT_MOUSE && self.mouse_override_active() {
			return InputState::released();
		}

		match input {
			UserInput::KeyCode(k) => self.get_single_state(&self.keys, k),
			UserInput::MouseButton(m) => self.get_single_state(&self.mouse, m),
			UserInput::Combined(c) => self.get_combination_state(c),
		}
	}
}

impl<'w, 's, TKeyMap> Input<'w, 's, TKeyMap>
where
	TKeyMap: SystemParam,
{
	fn mouse_override_active(&self) -> bool {
		*self.mouse_override != MouseOverride::Idle
	}

	fn matching_mouse_override(&self, action_key: ActionKey) -> Option<InputState> {
		let MouseOverride::Active {
			action,
			input_state,
			..
		} = self.mouse_override.deref()
		else {
			return None;
		};

		if &action_key != action {
			return None;
		}

		input_state.as_ref().copied()
	}

	fn get_combination_state(&self, combination: UserInputCombination<2>) -> InputState {
		let combination = combination.as_array().map(|c| match c {
			RawInput::KeyCode(k) => get_bevy_input_state(&self.keys, k),
			RawInput::MouseButton(m) => get_bevy_input_state(&self.mouse, m),
		});

		match combination {
			states::all_pressed!(just_pressed_1st, just_pressed_2nd) => InputState::Pressed {
				just_now: just_pressed_1st || just_pressed_2nd,
			},
			states::all_released!(just_released_1st, just_released_2nd) => InputState::Released {
				just_now: just_released_1st && just_released_2nd,
			},
			states::one_released! {just_pressed, just_released} => InputState::Released {
				just_now: just_released && !just_pressed,
			},
		}
	}

	fn get_single_state<T>(&self, keys: &Res<'w, ButtonInput<T>>, key: T) -> InputState
	where
		T: Copy + Eq + Hash + ThreadSafe + Into<RawInput>,
		TKeyMap: SystemParam<Item<'w, 's>: GetInput> + 'static,
	{
		use InputState::*;

		let state = get_bevy_input_state(keys, key);
		let combinations_ranked = ActionKey::iterator()
			.filter_map(|action| {
				let UserInput::Combined(combination) = self.key_map.get_input(action) else {
					return None;
				};

				if !combination.contains(&key.into()) {
					return None;
				}

				Some(Rank(self.get_input_state(action)))
			})
			.collect::<Sorted<_>>();

		if let Some(Rank(combination_state)) = combinations_ranked.first() {
			match (combination_state, state) {
				(Pressed { .. }, Pressed { just_now: true }) => {
					return Released { just_now: false };
				}
				(Pressed { just_now }, ..) => {
					return Released {
						just_now: *just_now,
					};
				}
				(Released { just_now: true }, Released { just_now: true }) => {
					return Released { just_now: false };
				}
				(Released { just_now }, Pressed { just_now: false }) => {
					return Pressed {
						just_now: *just_now,
					};
				}
				_ => {}
			};
		}

		state
	}
}

const LEFT_MOUSE: UserInput = UserInput::MouseButton(MouseButton::Left);

fn get_bevy_input_state<T>(input: &Res<ButtonInput<T>>, button: T) -> InputState
where
	T: Copy + Eq + Hash + ThreadSafe,
{
	if input.just_pressed(button) {
		return InputState::just_pressed();
	}

	if input.pressed(button) {
		return InputState::pressed();
	}

	if input.just_released(button) {
		return InputState::just_released();
	}

	InputState::released()
}

#[derive(Debug, PartialEq, Eq)]
struct Rank(InputState);

impl Rank {
	fn numeric(&self) -> u8 {
		let Rank(this) = self;

		match this {
			InputState::Pressed { just_now: false } => 0,
			InputState::Pressed { just_now: true } => 1,
			InputState::Released { just_now: true } => 2,
			InputState::Released { just_now: false } => 3,
		}
	}
}

impl PartialOrd for Rank {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Rank {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.numeric().cmp(&other.numeric())
	}
}

mod states {
	macro_rules! all_pressed {
		($just_pressed_1st:ident, $just_pressed_2nd:ident) => {
			[
				InputState::Pressed {
					just_now: $just_pressed_1st,
				},
				InputState::Pressed {
					just_now: $just_pressed_2nd,
				},
			]
		};
	}

	macro_rules! all_released {
		($just_released_1st:ident, $just_released_2nd:ident) => {
			[
				InputState::Released {
					just_now: $just_released_1st,
				},
				InputState::Released {
					just_now: $just_released_2nd,
				},
			]
		};
	}

	macro_rules! one_released {
		($just_pressed:ident, $just_released:ident) => {
			[
				InputState::Pressed {
					just_now: $just_pressed,
				},
				InputState::Released {
					just_now: $just_released,
				},
			] | [
				InputState::Released {
					just_now: $just_released,
				},
				InputState::Pressed {
					just_now: $just_pressed,
				},
			]
		};
	}

	pub(super) use all_pressed;
	pub(super) use all_released;
	pub(super) use one_released;
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::NestedMocks;
	use mockall::automock;
	use testing::{NestedMocks, SingleThreadedApp, set_input};

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl GetInput for _Map {
		fn get_input<TAction>(&self, action: TAction) -> UserInput
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.mock.get_input(action)
		}
	}

	#[derive(Default)]
	struct _Action(ActionKey);

	impl From<_Action> for ActionKey {
		fn from(_Action(action_key): _Action) -> Self {
			action_key
		}
	}

	type _Input<'w, 's> = Input<'w, 's, Res<'static, _Map>>;

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.init_resource::<MouseOverride>();
		app.init_resource::<ButtonInput<KeyCode>>();
		app.init_resource::<ButtonInput<MouseButton>>();

		app
	}

	mod single {
		use super::*;
		use test_case::test_case;

		#[test_case(KeyCode::KeyA; "key")]
		#[test_case(MouseButton::Right; "mouse button")]
		fn get_pressed<TInput>(user_input: TInput) -> Result<(), RunSystemError>
		where
			TInput: Into<UserInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(user_input.into());
			}));
			set_input!(app, pressed(user_input));

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::pressed(), state);
			Ok(())
		}

		#[test_case(KeyCode::KeyA; "key")]
		#[test_case(MouseButton::Right; "mouse button")]
		fn get_just_pressed<TInput>(user_input: TInput) -> Result<(), RunSystemError>
		where
			TInput: Into<UserInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(user_input.into());
			}));
			set_input!(app, just_pressed(user_input));

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::just_pressed(), state);
			Ok(())
		}

		#[test_case(KeyCode::KeyA; "key")]
		#[test_case(MouseButton::Right; "mouse button")]
		fn get_released<TInput>(user_input: TInput) -> Result<(), RunSystemError>
		where
			TInput: Into<UserInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(user_input.into());
			}));
			set_input!(app, released(user_input));

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test_case(KeyCode::KeyA; "key")]
		#[test_case(MouseButton::Right; "mouse button")]
		fn get_just_released<TInput>(user_input: TInput) -> Result<(), RunSystemError>
		where
			TInput: Into<UserInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(user_input.into());
			}));
			set_input!(app, just_released(user_input));

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::just_released(), state);
			Ok(())
		}
	}

	mod combo {
		use super::*;
		use mockall::predicate::eq;
		use test_case::test_case;

		#[derive(Debug)]
		#[allow(unused)]
		enum _Error {
			System(RunSystemError),
			Combo(InputRepeated),
		}

		impl From<RunSystemError> for _Error {
			fn from(value: RunSystemError) -> Self {
				Self::System(value)
			}
		}

		impl From<InputRepeated> for _Error {
			fn from(value: InputRepeated) -> Self {
				Self::Combo(value)
			}
		}

		macro_rules! set_input_state {
			($app:expr, $i:expr, $s:expr) => {
				match $s {
					InputState::Pressed { just_now: false } => {
						set_input!($app, pressed($i))
					}
					InputState::Pressed { just_now: true } => {
						set_input!($app, just_pressed($i))
					}
					InputState::Released { just_now: false } => {
						set_input!($app, released($i))
					}
					InputState::Released { just_now: true } => {
						set_input!($app, just_released($i))
					}
				}
			};
		}

		macro_rules! set_combo_state {
			($app:expr, $c:expr) => {{
				for (c, s) in $c {
					match c {
						RawInput::KeyCode(k) => set_input_state!($app, k, s),
						RawInput::MouseButton(m) => set_input_state!($app, m, s),
					}
				}
			}};
		}

		#[test_case([
			RawInput::KeyCode(KeyCode::KeyA),
			RawInput::KeyCode(KeyCode::KeyB),
		]; "key code")]
		#[test_case([
			RawInput::MouseButton(MouseButton::Left),
			RawInput::MouseButton(MouseButton::Right),
		]; "mouse buttons")]
		#[test_case([
			RawInput::KeyCode(KeyCode::KeyA),
			RawInput::MouseButton(MouseButton::Right),
		]; "combined")]
		fn get_pressed(combo: [RawInput; 2]) -> Result<(), _Error> {
			let action_combo = UserInput::Combined(UserInputCombination::try_from(combo)?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(action_combo);
			}));
			set_combo_state!(app, combo.map(|c| (c, InputState::pressed())));

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::pressed(), state);
			Ok(())
		}

		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::just_pressed()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::just_pressed()),
		]; "all just pressed")]
		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::pressed()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::just_pressed()),
		]; "last just pressed")]
		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::just_pressed()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::pressed()),
		]; "first just pressed")]
		fn get_just_pressed(combo: [(RawInput, InputState); 2]) -> Result<(), _Error> {
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(combo.map(|(i, ..)| i))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(action_combo);
			}));
			set_combo_state!(app, combo);

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::just_pressed(), state);
			Ok(())
		}

		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::released()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::released()),
		]; "all released")]
		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::released()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::just_released()),
		]; "released and just released")]
		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::just_released()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::released()),
		]; "just released and released")]
		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::just_pressed()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::just_released()),
		]; "just pressed and just released")]
		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::just_released()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::just_pressed()),
		]; "just released and just pressed")]
		fn get_released(combo: [(RawInput, InputState); 2]) -> Result<(), _Error> {
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(combo.map(|(i, ..)| i))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(action_combo);
			}));
			set_combo_state!(app, combo);

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::just_released()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::just_released()),
		]; "all just released")]
		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::pressed()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::just_released()),
		]; "pressed and just released")]
		#[test_case([
			(RawInput::KeyCode(KeyCode::KeyA), InputState::just_released()),
			(RawInput::KeyCode(KeyCode::KeyB), InputState::pressed()),
		]; "just released and pressed")]
		fn get_just_release(combo: [(RawInput, InputState); 2]) -> Result<(), _Error> {
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(combo.map(|(i, ..)| i))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(action_combo);
			}));
			set_combo_state!(app, combo);

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::just_released(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], MouseButton::Middle; "mouse button")]
		fn single_released_when_related_combo_pressed<T>(
			keys: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(keys.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(keys[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(action_combo);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, pressed(keys[0]));
			set_input!(app, pressed(keys[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], MouseButton::Middle; "mouse button")]
		fn single_just_released_when_related_combo_just_pressed<T>(
			keys: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(keys.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(keys[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(action_combo);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, pressed(keys[0]));
			set_input!(app, just_pressed(keys[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::just_released(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], MouseButton::Middle; "mouse button")]
		fn single_released_when_related_combo_just_pressed<T>(
			keys: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(keys.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(keys[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(action_combo);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, just_pressed(keys[0]));
			set_input!(app, pressed(keys[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right, MouseButton::Back], MouseButton::Middle; "mouse button")]
		fn single_pressed_when_unrelated_combo_pressed<T>(
			keys: [T; 3],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let [key, comb_a, comb_b] = keys;
			let action_combo = UserInput::Combined(UserInputCombination::try_from(
				[comb_a, comb_b].map(T::into),
			)?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(key.into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(action_combo);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, pressed(keys[0]));
			set_input!(app, pressed(keys[1]));
			set_input!(app, pressed(keys[2]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::pressed(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], MouseButton::Middle; "mouse button")]
		fn single_just_pressed_when_related_combo_just_released<T>(
			keys: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(keys.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(keys[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(action_combo);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, pressed(keys[0]));
			set_input!(app, just_released(keys[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::just_pressed(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], MouseButton::Middle; "mouse button")]
		fn single_released_when_causing_related_combo_to_be_released<T>(
			keys: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(keys.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(keys[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(action_combo);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, just_released(keys[0]));
			set_input!(app, pressed(keys[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], MouseButton::Middle; "mouse button")]
		fn single_just_released_when_related_combo_already_released<T>(
			keys: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(keys.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(keys[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(action_combo);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, just_released(keys[0]));
			set_input!(app, released(keys[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::just_released(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], MouseButton::Middle; "mouse button")]
		fn single_just_pressed_when_related_combo_already_released<T>(
			keys: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let action_combo =
				UserInput::Combined(UserInputCombination::try_from(keys.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(keys[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(action_combo);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, just_pressed(keys[0]));
			set_input!(app, released(keys[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::just_pressed(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], [KeyCode::KeyA, KeyCode::KeyC], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], [MouseButton::Left, MouseButton::Back], MouseButton::Middle; "mouse button")]
		fn prioritize_pressed_over_just_pressed_among_different_combos<T>(
			a: [T; 2],
			b: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let combo_a = UserInput::Combined(UserInputCombination::try_from(a.map(T::into))?);
			let combo_b = UserInput::Combined(UserInputCombination::try_from(b.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(a[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(combo_a);
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Forward)))
					.return_const(combo_b);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, pressed(a[0]));
			set_input!(app, pressed(a[1]));
			set_input!(app, just_pressed(b[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], [KeyCode::KeyA, KeyCode::KeyC], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], [MouseButton::Left, MouseButton::Back], MouseButton::Middle; "mouse button")]
		fn prioritize_just_released_over_released_among_different_combos<T>(
			a: [T; 2],
			b: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let combo_a = UserInput::Combined(UserInputCombination::try_from(a.map(T::into))?);
			let combo_b = UserInput::Combined(UserInputCombination::try_from(b.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(a[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(combo_a);
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Forward)))
					.return_const(combo_b);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, pressed(a[0]));
			set_input!(app, just_released(a[1]));
			set_input!(app, released(b[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::just_pressed(), state);
			Ok(())
		}

		#[test_case([KeyCode::KeyA, KeyCode::KeyB], [KeyCode::KeyA, KeyCode::KeyC], KeyCode::ShiftLeft; "key code")]
		#[test_case([MouseButton::Left, MouseButton::Right], [MouseButton::Left, MouseButton::Back], MouseButton::Middle; "mouse button")]
		fn prioritize_pressed_over_released_among_different_combos<T>(
			a: [T; 2],
			b: [T; 2],
			rest: T,
		) -> Result<(), _Error>
		where
			T: Into<UserInput> + Into<RawInput> + Copy + Eq + Hash + ThreadSafe,
		{
			let combo_a = UserInput::Combined(UserInputCombination::try_from(a.map(T::into))?);
			let combo_b = UserInput::Combined(UserInputCombination::try_from(b.map(T::into))?);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Left)))
					.return_const::<UserInput>(a[0].into());
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Right)))
					.return_const(combo_a);
				mock.expect_get_input::<ActionKey>()
					.with(eq(ActionKey::Movement(MovementKey::Forward)))
					.return_const(combo_b);
				mock.expect_get_input::<ActionKey>()
					.return_const::<UserInput>(rest.into());
			}));
			set_input!(app, pressed(a[0]));
			set_input!(app, pressed(a[1]));
			set_input!(app, released(b[1]));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(_Action(ActionKey::Movement(MovementKey::Left)))
			})?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}
	}

	mod mouse_override {
		use super::*;
		use test_case::test_case;
		use testing::fake_entity;

		const LEFT_MOUSE: MouseButton = MouseButton::Left;

		#[test]
		fn ignore_left_mouse_button_when_mouse_override_active() -> Result<(), RunSystemError> {
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(LEFT_MOUSE);
			}));
			app.insert_resource(MouseOverride::Active {
				panel: fake_entity!(42),
				action: ActionKey::from(HandSlot::Left),
				input_state: None,
			});
			set_input!(app, pressed(LEFT_MOUSE));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(ActionKey::from(HandSlot::Right))
			})?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test]
		fn use_left_mouse_button_when_mouse_override_idle() -> Result<(), RunSystemError> {
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(LEFT_MOUSE);
			}));
			app.insert_resource(MouseOverride::Idle);
			set_input!(app, pressed(LEFT_MOUSE));

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::pressed(), state);
			Ok(())
		}

		#[test_case(InputState::just_pressed(); "just pressed")]
		#[test_case(InputState::pressed(); "pressed")]
		#[test_case(InputState::just_released(); "just released")]
		fn return_override_key_input_state(input_state: InputState) -> Result<(), RunSystemError> {
			let action = ActionKey::from(HandSlot::Left);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>().never();
			}));
			app.insert_resource(MouseOverride::Active {
				panel: fake_entity!(123),
				action,
				input_state: Some(input_state),
			});

			let state = app
				.world_mut()
				.run_system_once(move |input: _Input| input.get_input_state(action))?;

			assert_eq!(input_state, state);
			Ok(())
		}
	}
}
