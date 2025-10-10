use crate::{Input, KeyBind, Rebinding};
use bevy::prelude::*;
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{handles_input::UpdateKey, thread_safe::ThreadSafe},
};

impl<TAction> KeyBind<Rebinding<TAction>>
where
	TAction: Copy + ThreadSafe + Into<ActionKey> + Into<UserInput>,
{
	pub(crate) fn rebind_apply<TMap>(
		mut map: ResMut<TMap>,
		input: Res<ButtonInput<UserInput>>,
		rebinds: Query<&Self>,
	) where
		TMap: UpdateKey + Resource,
	{
		for input in input.get_just_pressed() {
			for KeyBind(Rebinding(Input { action, .. })) in &rebinds {
				map.update_key(*action, *input);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Action;

	impl From<_Action> for UserInput {
		fn from(_: _Action) -> Self {
			panic!("NOT USED")
		}
	}

	impl From<_Action> for ActionKey {
		fn from(_: _Action) -> Self {
			panic!("NOT USED")
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl UpdateKey for _Map {
		fn update_key<TAction>(&mut self, key: TAction, user_input: UserInput)
		where
			TAction: Copy + Into<ActionKey> + Into<UserInput> + 'static,
		{
			self.mock.update_key(key, user_input)
		}
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<ButtonInput<UserInput>>();
		app.insert_resource(map);
		app.add_systems(Update, KeyBind::<Rebinding<_Action>>::rebind_apply::<_Map>);

		app
	}

	#[test]
	fn apply_rebind() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_update_key()
				.with(eq(_Action), eq(UserInput::KeyCode(KeyCode::KeyB)))
				.times(1)
				.return_const(());
		}));
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::KeyCode(KeyCode::KeyB));

		app.update();
	}

	#[test]
	fn do_not_apply_rebind_if_not_just_pressed() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_update_key::<_Action>().never().return_const(());
		}));
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: UserInput::KeyCode(KeyCode::KeyA),
		})));
		let mut input = app.world_mut().resource_mut::<ButtonInput<UserInput>>();
		input.press(UserInput::KeyCode(KeyCode::KeyB));
		input.clear_just_pressed(UserInput::KeyCode(KeyCode::KeyB));

		app.update();
	}
}
