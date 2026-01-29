use crate::components::quickbar_panel::QuickbarPanel;
use bevy::prelude::*;
use common::{
	tools::action_key::ActionKey,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};

impl QuickbarPanel {
	pub(crate) fn add_input_control<TActionKeyButton>(
		trigger: On<Add, Self>,
		mut commands: ZyheedaCommands,
		panels: Query<&Self>,
	) where
		TActionKeyButton: Component + From<ActionKey>,
	{
		let entity = trigger.entity;
		let Ok(Self { key, .. }) = panels.get(entity) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(TActionKeyButton::from(ActionKey::from(*key)));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::PanelState;
	use common::tools::action_key::{ActionKey, slot::PlayerSlot};
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _ActionKeyButton(ActionKey);

	impl From<ActionKey> for _ActionKeyButton {
		fn from(key: ActionKey) -> Self {
			Self(key)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(QuickbarPanel::add_input_control::<_ActionKeyButton>);

		app
	}

	#[test]
	fn add_ui_input_primer() {
		let mut app = setup();
		let entity = app.world_mut().spawn(QuickbarPanel {
			key: PlayerSlot::UPPER_L,
			state: PanelState::Empty,
		});

		assert_eq!(
			Some(&_ActionKeyButton(ActionKey::from(PlayerSlot::UPPER_L))),
			entity.get::<_ActionKeyButton>()
		);
	}
}
