use crate::system_params::input::Input;
use bevy::ecs::system::SystemParam;
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_input::{GetAllInputs, GetInput},
		iteration::{Iter as VariationsIter, IterFinite},
	},
};

impl<TKeyMap> GetAllInputs for Input<'_, '_, TKeyMap>
where
	TKeyMap: SystemParam + 'static,
	Self: GetInput,
{
	fn get_all_inputs(&self) -> impl Iterator<Item = (ActionKey, UserInput)> {
		Iter {
			input: self,
			actions: ActionKey::iterator(),
		}
	}
}

struct Iter<'a, TInput, TAction> {
	input: &'a TInput,
	actions: VariationsIter<TAction>,
}

impl<'a, TInput, TAction> Iterator for Iter<'a, TInput, TAction>
where
	TInput: GetInput,
	TAction: Into<ActionKey> + IterFinite + 'static,
{
	type Item = (TAction, UserInput);

	fn next(&mut self) -> Option<Self::Item> {
		let action_key = self.actions.next()?;
		Some((action_key, self.input.get_input(action_key)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::input::keyboard::KeyCode;
	use macros::simple_mock;
	use testing::Mock;

	simple_mock! {
		_Input {}
		impl GetInput for _Input {
			fn get_input<TAction>(&self, action: TAction) -> UserInput
			where
				TAction: Into<ActionKey> + 'static;
		}
	}

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Action;

	impl IterFinite for _Action {
		fn iterator() -> VariationsIter<Self> {
			VariationsIter(Some(_Action))
		}

		fn next(_: &VariationsIter<Self>) -> Option<Self> {
			None
		}
	}

	impl From<_Action> for ActionKey {
		fn from(_: _Action) -> Self {
			panic!("DO NOT USE")
		}
	}

	#[test]
	fn all_inputs() {
		let input = &Mock_Input::new_mock(|mock| {
			mock.expect_get_input::<_Action>()
				.times(1)
				.return_const(UserInput::KeyCode(KeyCode::AltLeft));
		});
		let iter = Iter {
			input,
			actions: _Action::iterator(),
		};

		assert_eq!(
			vec![(_Action, UserInput::KeyCode(KeyCode::AltLeft))],
			iter.collect::<Vec<_>>()
		);
	}
}
