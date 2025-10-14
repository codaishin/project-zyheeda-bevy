use crate::{Input, KeyBind, Rebinding};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::ActionKey,
	traits::{
		handles_input::{GetRawUserInput, RawInputState, UpdateKey},
		thread_safe::ThreadSafe,
	},
};

impl<TAction> KeyBind<Rebinding<TAction>>
where
	TAction: Copy + ThreadSafe + Into<ActionKey>,
{
	pub(crate) fn rebind_apply<TInputMut>(
		mut input: StaticSystemParam<TInputMut>,
		rebinds: Query<&Self>,
	) where
		for<'w, 's> TInputMut: SystemParam<Item<'w, 's>: GetRawUserInput + UpdateKey>,
	{
		let inputs = input
			.get_raw_user_input(RawInputState::JustPressed)
			.collect::<Vec<_>>();

		for user_input in inputs {
			for KeyBind(Rebinding(Input { action, .. })) in &rebinds {
				input.update_key(*action, user_input);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::user_input::UserInput;
	use std::{any::type_name, collections::HashMap};
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
		updated: Vec<(&'static str, UserInput)>,
		raw_input: HashMap<RawInputState, Vec<UserInput>>,
	}

	impl UpdateKey for _Input {
		fn update_key<TAction>(&mut self, _: TAction, user_input: UserInput)
		where
			TAction: Copy + Into<ActionKey> + 'static,
		{
			self.updated.push((type_name::<TAction>(), user_input));
		}
	}

	impl GetRawUserInput for _Input {
		fn get_raw_user_input(&self, state: RawInputState) -> impl Iterator<Item = UserInput> {
			self.raw_input
				.get(&state)
				.cloned()
				.unwrap_or(vec![])
				.into_iter()
		}
	}

	fn setup(map: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.add_systems(
			Update,
			KeyBind::<Rebinding<_Action>>::rebind_apply::<ResMut<_Input>>,
		);

		app
	}

	#[test]
	fn apply_rebind() {
		let mut app = setup(_Input {
			raw_input: HashMap::from([(
				RawInputState::JustPressed,
				vec![UserInput::KeyCode(KeyCode::KeyB)],
			)]),
			..default()
		});
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		app.update();

		assert_eq!(
			vec![(type_name::<_Action>(), UserInput::KeyCode(KeyCode::KeyB))],
			app.world().resource::<_Input>().updated,
		);
	}

	#[test]
	fn do_not_apply_rebind_if_not_just_pressed() {
		let mut app = setup(_Input {
			raw_input: HashMap::from([(RawInputState::JustPressed, vec![])]),
			..default()
		});
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));

		app.update();

		assert!(app.world().resource::<_Input>().updated.is_empty());
	}
}
