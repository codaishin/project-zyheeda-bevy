use crate::{Input, KeyBind, Rebinding};
use bevy::prelude::*;
use common::traits::{handles_settings::UpdateKey, thread_safe::ThreadSafe};
use std::hash::Hash;

impl<TAction, TInput> KeyBind<Rebinding<TAction, TInput>>
where
	TAction: Copy + ThreadSafe,
	TInput: Copy + Eq + Hash + ThreadSafe,
{
	pub(crate) fn rebind_apply<TMap>(
		mut map: ResMut<TMap>,
		input: Res<ButtonInput<TInput>>,
		rebinds: Query<&Self>,
	) where
		TMap: UpdateKey<TAction, TInput> + Resource,
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

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Input {
		A,
		B,
	}

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl UpdateKey<_Action, _Input> for _Map {
		fn update_key(&mut self, key: _Action, user_input: _Input) {
			self.mock.update_key(key, user_input)
		}
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<ButtonInput<_Input>>();
		app.insert_resource(map);
		app.add_systems(
			Update,
			KeyBind::<Rebinding<_Action, _Input>>::rebind_apply::<_Map>,
		);

		app
	}

	#[test]
	fn apply_rebind() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_update_key()
				.with(eq(_Action), eq(_Input::B))
				.times(1)
				.return_const(());
		}));
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: _Input::A,
		})));
		app.world_mut()
			.resource_mut::<ButtonInput<_Input>>()
			.press(_Input::B);

		app.update();
	}

	#[test]
	fn do_not_apply_rebind_if_not_just_pressed() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_update_key().never().return_const(());
		}));
		app.world_mut().spawn(KeyBind(Rebinding(Input {
			action: _Action,
			input: _Input::A,
		})));
		let mut input = app.world_mut().resource_mut::<ButtonInput<_Input>>();
		input.press(_Input::B);
		input.clear_just_pressed(_Input::B);

		app.update();
	}
}
