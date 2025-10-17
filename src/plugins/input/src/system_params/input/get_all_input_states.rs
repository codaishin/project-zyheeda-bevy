use crate::system_params::input::Input;
use bevy::ecs::system::SystemParam;
use common::{
	tools::action_key::ActionKey,
	traits::{
		handles_input::{GetAllInputStates, GetInputState, InputState},
		iteration::{Iter as VariationsIter, IterFinite},
	},
};

impl<TKeyMap> GetAllInputStates for Input<'_, '_, TKeyMap>
where
	TKeyMap: SystemParam + 'static,
	Self: GetInputState,
{
	fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
	where
		TAction: Into<ActionKey> + IterFinite + 'static,
	{
		Iter {
			input: self,
			actions: TAction::iterator(),
		}
	}
}

struct Iter<'a, TInput, TAction> {
	input: &'a TInput,
	actions: VariationsIter<TAction>,
}

impl<'a, TInput, TAction> Iterator for Iter<'a, TInput, TAction>
where
	TInput: GetInputState,
	TAction: Into<ActionKey> + IterFinite + 'static,
{
	type Item = (TAction, InputState);

	fn next(&mut self) -> Option<Self::Item> {
		let action_key = self.actions.next()?;
		Some((action_key, self.input.get_input_state(action_key)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use macros::simple_mock;
	use testing::Mock;

	simple_mock! {
		_Input {}
		impl GetInputState for _Input {
			fn get_input_state<TAction>(&self, action: TAction) -> InputState
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
	fn all_input_states() {
		let input = &Mock_Input::new_mock(|mock| {
			mock.expect_get_input_state::<_Action>()
				.times(1)
				.return_const(InputState::just_pressed());
		});
		let iter = Iter {
			input,
			actions: _Action::iterator(),
		};

		assert_eq!(
			vec![(_Action, InputState::just_pressed())],
			iter.collect::<Vec<_>>(),
		);
	}
}
