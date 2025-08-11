use crate::components::quickbar_panel::QuickbarPanel;
use bevy::prelude::*;
use common::{
	components::ui_input_primer::UiInputPrimer,
	tools::action_key::{slot::PlayerSlot, user_input::UserInput},
	traits::{accessors::get::TryApplyOn, key_mappings::GetInput},
	zyheeda_commands::ZyheedaCommands,
};

impl QuickbarPanel {
	pub(crate) fn add_quickbar_primer<TMap>(
		mut commands: ZyheedaCommands,
		map: Res<TMap>,
		panels: Query<(Entity, &Self), Added<Self>>,
	) where
		TMap: GetInput<PlayerSlot, TInput = UserInput> + Resource,
	{
		for (entity, panel) in &panels {
			let input = map.get_input(panel.key);
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(UiInputPrimer::from(input));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::PanelState;
	use common::{components::ui_input_primer::UiInputPrimer, tools::action_key::slot::Side};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl GetInput<PlayerSlot> for _Map {
		type TInput = UserInput;

		fn get_input(&self, value: PlayerSlot) -> UserInput {
			self.mock.get_input(value)
		}
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.add_systems(Update, QuickbarPanel::add_quickbar_primer::<_Map>);

		app
	}

	#[test]
	fn add_ui_input_primer() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input()
				.times(1)
				.with(eq(PlayerSlot::Upper(Side::Left)))
				.return_const(UserInput::from(MouseButton::Right));
		}));
		let entity = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::Upper(Side::Left),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		assert_eq!(
			Some(&UiInputPrimer::from(UserInput::from(MouseButton::Right))),
			app.world().entity(entity).get::<UiInputPrimer>()
		);
	}

	#[test]
	fn do_not_add_twice() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input()
				.times(1)
				.return_const(UserInput::from(MouseButton::Right));
		}));
		app.world_mut().spawn(QuickbarPanel {
			key: PlayerSlot::Upper(Side::Left),
			state: PanelState::Empty,
		});

		app.update();
		app.update();
	}
}
