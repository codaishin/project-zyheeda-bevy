use crate::system_params::input::Input;
use bevy::ecs::system::SystemParam;
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::handles_input::GetInput,
};

impl<'w, 's, TKeyMap> GetInput for Input<'w, 's, TKeyMap>
where
	TKeyMap: SystemParam<Item<'w, 's>: GetInput> + 'static,
{
	fn get_input<TAction>(&self, action: TAction) -> UserInput
	where
		TAction: Into<ActionKey> + 'static,
	{
		self.key_map.get_input(action)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::tools::action_key::user_input::UserInput;
	use macros::NestedMocks;
	use mockall::automock;
	use testing::{NestedMocks, SingleThreadedApp};

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

	struct _Action;

	impl From<_Action> for ActionKey {
		fn from(_: _Action) -> Self {
			panic!("DO NOT USE")
		}
	}

	type _Input<'w, 's> = Input<'w, 's, Res<'static, _Map>>;

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
			mock.expect_get_input::<_Action>()
				.return_const(UserInput::KeyCode(KeyCode::ArrowUp));
		}));

		let user_input = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input(_Action))?;

		assert_eq!(UserInput::KeyCode(KeyCode::ArrowUp), user_input);
		Ok(())
	}
}
