use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	tools::action_key::{ActionKey, save_key::SaveKey, user_input::UserInput},
	traits::key_mappings::JustPressed,
};

impl<T> TriggerQuickSave for T where T: Resource + JustPressed<ActionKey> {}

pub(crate) trait TriggerQuickSave: Resource + JustPressed<ActionKey> {
	fn trigger_quick_save(
		key_map: Res<Self>,
		input: Res<ButtonInput<UserInput>>,
		mut game_state: ResMut<NextState<GameState>>,
	) {
		if !key_map.just_pressed(&input).any(is_quick_save) {
			return;
		}

		game_state.set(GameState::Saving);
	}
}

fn is_quick_save(action: ActionKey) -> bool {
	action == ActionKey::Save(SaveKey::QuickSave)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::state::app::StatesPlugin;
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::automock;

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl JustPressed<ActionKey> for _Map {
		fn just_pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = ActionKey> {
			self.mock.just_pressed(input)
		}
	}

	fn setup(map: _Map, input: ButtonInput<UserInput>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<GameState>();
		app.insert_resource(input);
		app.insert_resource(map);
		app.add_systems(Update, _Map::trigger_quick_save);

		app
	}

	#[test]
	fn trigger_saving() {
		let map = _Map::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::once(ActionKey::Save(SaveKey::QuickSave))));
		});
		let mut app = setup(map, ButtonInput::default());

		app.update();

		assert!(matches!(
			app.world().get_resource::<NextState<GameState>>(),
			Some(NextState::Pending(GameState::Saving))
		));
	}

	#[test]
	fn do_not_trigger_saving_when_no_quick_save() {
		let map = _Map::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::once(ActionKey::Save(SaveKey::QuickLoad))));
		});
		let mut app = setup(map, ButtonInput::default());

		app.update();

		assert!(matches!(
			app.world().get_resource::<NextState<GameState>>(),
			Some(NextState::Unchanged)
		));
	}

	#[test]
	fn call_key_map_with_correct_arguments() {
		let map = _Map::new().with_mock(|mock| {
			mock.expect_just_pressed().times(1).returning(|input| {
				assert!(input.just_pressed(UserInput::MouseButton(MouseButton::Forward)));
				Box::new(std::iter::empty())
			});
		});
		let mut input = ButtonInput::default();
		input.press(UserInput::MouseButton(MouseButton::Forward));
		let mut app = setup(map, input);

		app.update();
	}
}
