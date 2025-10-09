use crate::components::quickbar_panel::QuickbarPanel;
use bevy::prelude::*;
use common::{
	components::ui_input_primer::UiInputPrimer,
	traits::{accessors::get::TryApplyOn, key_mappings::GetInput},
	zyheeda_commands::ZyheedaCommands,
};

impl QuickbarPanel {
	pub(crate) fn add_quickbar_primer<TMap>(
		trigger: Trigger<OnAdd, Self>,
		mut commands: ZyheedaCommands,
		map: Res<TMap>,
		panels: Query<&Self>,
	) where
		TMap: GetInput + Resource,
	{
		let entity = trigger.target();
		let Ok(Self { key, .. }) = panels.get(entity) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(UiInputPrimer::from(map.get_input(*key)));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::PanelState;
	use common::{
		components::ui_input_primer::UiInputPrimer,
		tools::action_key::{
			ActionKey,
			slot::{PlayerSlot, Side},
			user_input::UserInput,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl GetInput for _Map {
		fn get_input<TAction>(&self, value: TAction) -> UserInput
		where
			TAction: Copy + Into<ActionKey> + Into<UserInput> + 'static,
		{
			self.mock.get_input(value)
		}
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.add_observer(QuickbarPanel::add_quickbar_primer::<_Map>);

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
		let entity = app.world_mut().spawn(QuickbarPanel {
			key: PlayerSlot::Upper(Side::Left),
			state: PanelState::Empty,
		});

		assert_eq!(
			Some(&UiInputPrimer::from(UserInput::from(MouseButton::Right))),
			entity.get::<UiInputPrimer>()
		);
	}
}
