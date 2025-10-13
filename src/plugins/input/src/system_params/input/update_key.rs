use crate::system_params::input::Input;
use bevy::prelude::*;
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::handles_input::UpdateKey,
};

impl<'w, 's, TKeyMap> UpdateKey for Input<'w, 's, ResMut<'static, TKeyMap>>
where
	TKeyMap: Resource + UpdateKey,
{
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey> + 'static,
	{
		self.key_map.update_key(action, input);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::action_key::user_input::UserInput;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl UpdateKey for _Map {
		fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
		where
			TAction: Copy + Into<ActionKey> + 'static,
		{
			self.mock.update_key(action, input);
		}
	}

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Action;

	impl From<_Action> for ActionKey {
		fn from(_: _Action) -> Self {
			panic!("DO NOT USE")
		}
	}

	type _Input<'w, 's> = Input<'w, 's, ResMut<'static, _Map>>;

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.init_resource::<ButtonInput<KeyCode>>();
		app.init_resource::<ButtonInput<MouseButton>>();

		app
	}

	#[test]
	fn get_input() -> Result<(), RunSystemError> {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_update_key()
				.times(1)
				.with(
					eq(_Action),
					eq(UserInput::MouseButton(MouseButton::Forward)),
				)
				.return_const(());
		}));

		app.world_mut().run_system_once(|mut input: _Input| {
			input.update_key(_Action, UserInput::MouseButton(MouseButton::Forward))
		})
	}
}
