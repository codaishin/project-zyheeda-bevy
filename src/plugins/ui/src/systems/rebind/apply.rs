use crate::{Input, KeyBind, Rebinding};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::prelude::*;
use zyheeda_core::collections::ordered::OrderedSet;

impl<TAction> KeyBind<Rebinding<TAction>>
where
	TAction: Copy + ThreadSafe + Into<ActionKey>,
{
	pub(crate) fn rebind_apply<TInputMut>(
		mut input: StaticSystemParam<TInputMut>,
		rebinds: Query<&Self>,
		mut cache: Local<OrderedSet<RawInput>>,
	) where
		for<'w, 's> TInputMut: SystemParam<Item<'w, 's>: GetRawUserInput + UpdateKey>,
	{
		if rebinds.is_empty() {
			return;
		}

		let mut is_holding = false;

		for input in input.get_raw_user_input(RawInputState::Held) {
			is_holding = true;
			cache.insert(input);
		}

		if is_holding {
			return;
		}

		let mut inputs = OrderedSet::default();

		std::mem::swap(&mut inputs, &mut cache);

		let user_input = match UserInputCombination::try_from_set(inputs) {
			Ok(comb) => UserInput::Combined(comb),
			Err(raw) => match raw.into_iter().next() {
				Some(raw) => UserInput::from(raw),
				None => return,
			},
		};

		for KeyBind(Rebinding(Input { action, .. })) in &rebinds {
			input.update_key(*action, user_input);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::{any::type_name, collections::HashMap, sync::OnceLock};
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Action;

	impl From<_Action> for ActionKey {
		fn from(_: _Action) -> Self {
			panic!("NOT USED")
		}
	}

	#[derive(Resource, Default)]
	struct _Input {
		called: OnceLock<()>,
		updated: Vec<(&'static str, UserInput)>,
		raw_input: HashMap<RawInputState, Vec<RawInput>>,
	}

	impl _Input {
		fn called_raw_user_input(&self) -> bool {
			self.called.get().is_some()
		}
	}

	impl UpdateKey for _Input {
		fn update_key<TAction, TInput>(&mut self, _: TAction, user_input: TInput)
		where
			TAction: Copy + Into<ActionKey> + 'static,
			TInput: Into<UserInput>,
		{
			self.updated
				.push((type_name::<TAction>(), user_input.into()));
		}
	}

	impl GetRawUserInput for _Input {
		fn get_raw_user_input(&self, state: RawInputState) -> impl Iterator<Item = RawInput> {
			_ = self.called.set(());
			self.raw_input
				.get(&state)
				.cloned()
				.unwrap_or(vec![])
				.into_iter()
		}
	}

	macro_rules! set_held_input {
		($app:expr, $input:expr $(,)?) => {
			$app.insert_resource(_Input {
				raw_input: HashMap::from([(RawInputState::Held, Vec::from($input))]),
				..default()
			})
		};
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_Input::default());
		app.add_systems(
			Update,
			KeyBind::<Rebinding<_Action>>::rebind_apply::<ResMut<_Input>>,
		);

		app
	}

	#[test]
	fn apply_rebind() {
		let mut app = setup();
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		set_held_input!(app, [RawInput::KeyCode(KeyCode::KeyB)]);
		app.update();
		set_held_input!(app, []);
		app.update();

		assert_eq!(
			vec![(type_name::<_Action>(), UserInput::KeyCode(KeyCode::KeyB))],
			app.world().resource::<_Input>().updated,
		);
	}

	#[test]
	fn apply_rebind_when_held_same_key_for_2_frames() {
		let mut app = setup();
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		set_held_input!(app, [RawInput::KeyCode(KeyCode::KeyB)]);
		app.update();
		app.update();
		set_held_input!(app, []);
		app.update();

		assert_eq!(
			vec![(type_name::<_Action>(), UserInput::KeyCode(KeyCode::KeyB))],
			app.world().resource::<_Input>().updated,
		);
	}

	#[test]
	fn apply_combination() -> Result<(), InputRepeatedError> {
		let mut app = setup();
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		set_held_input!(
			app,
			[
				RawInput::KeyCode(KeyCode::KeyB),
				RawInput::KeyCode(KeyCode::KeyC),
			],
		);
		app.update();
		set_held_input!(app, []);
		app.update();

		assert_eq!(
			vec![(
				type_name::<_Action>(),
				UserInput::Combined(UserInputCombination::try_from([
					RawInput::KeyCode(KeyCode::KeyB),
					RawInput::KeyCode(KeyCode::KeyC)
				])?)
			)],
			app.world().resource::<_Input>().updated,
		);
		Ok(())
	}

	#[test]
	fn apply_combo_rebind_across_frames() -> Result<(), InputRepeatedError> {
		let mut app = setup();
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		set_held_input!(app, [RawInput::KeyCode(KeyCode::KeyB)]);
		app.update();
		set_held_input!(app, [RawInput::KeyCode(KeyCode::KeyC)]);
		app.update();
		set_held_input!(app, []);
		app.update();

		assert_eq!(
			vec![(
				type_name::<_Action>(),
				UserInput::Combined(UserInputCombination::try_from([
					RawInput::KeyCode(KeyCode::KeyB),
					RawInput::KeyCode(KeyCode::KeyC)
				])?)
			)],
			app.world().resource::<_Input>().updated,
		);
		Ok(())
	}

	#[test]
	fn do_not_apply_rebind_if_nothing_pressed() {
		let mut app = setup();
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		app.update();
		app.update();

		assert!(app.world().resource::<_Input>().updated.is_empty());
	}

	#[test]
	fn do_not_apply_rebind_if_still_pressed() {
		let mut app = setup();
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		set_held_input!(app, [RawInput::KeyCode(KeyCode::KeyB)]);
		app.update();
		app.update();

		assert!(app.world().resource::<_Input>().updated.is_empty());
	}

	#[test]
	fn flush_when_nothing_pressed() {
		let mut app = setup();
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		set_held_input!(app, [RawInput::KeyCode(KeyCode::KeyB)]);
		app.update();
		set_held_input!(app, []);
		app.update();
		set_held_input!(app, [RawInput::KeyCode(KeyCode::KeyC)]);
		app.update();
		set_held_input!(app, []);
		app.update();

		assert_eq!(
			vec![(type_name::<_Action>(), UserInput::KeyCode(KeyCode::KeyC))],
			app.world().resource::<_Input>().updated,
		);
	}

	#[test]
	fn do_nothing_if_no_rebids() {
		let mut app = setup();

		app.update();

		assert!(!app.world().resource::<_Input>().called_raw_user_input());
	}
}
