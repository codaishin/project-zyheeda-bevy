use crate::components::{Dad, KeyedPanel};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{Get, TryApplyOn, View},
		handles_player::PlayerEntity,
	},
	zyheeda_commands::ZyheedaCommands,
};

pub fn drag_item<TPlayer>(
	mut commands: ZyheedaCommands,
	player: StaticSystemParam<TPlayer>,
	panels: Query<(&Interaction, &KeyedPanel)>,
) where
	TPlayer: for<'w, 's> SystemParam<Item<'w, 's>: View<PlayerEntity>>,
{
	let Some(player) = player.view() else {
		return;
	};
	let Some(entity) = commands.get(&player) else {
		return;
	};
	let Some((.., panel)) = panels.iter().find(is_pressed) else {
		return;
	};

	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(Dad(panel.0));
	});
}

fn is_pressed((interaction, _): &(&Interaction, &KeyedPanel)) -> bool {
	Interaction::Pressed == **interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::Dad,
		testing::{_Player, _PlayerParam},
	};
	use bevy::app::{App, Update};
	use common::{CommonPlugin, tools::action_key::slot::SlotKey};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.add_systems(Update, drag_item::<_PlayerParam>);

		app
	}

	#[test]
	fn drag_panel_on_pressed() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Player).id();
		app.world_mut()
			.spawn((Interaction::Pressed, KeyedPanel::from(SlotKey(42))));

		app.update();

		assert_eq!(
			Some(&Dad::from(SlotKey(42))),
			app.world().entity(agent).get::<Dad>()
		);
	}

	#[test]
	fn drag_panel_on_pressed_when_multiple_panels_exist() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Player).id();
		app.world_mut()
			.spawn((Interaction::Pressed, KeyedPanel::from(SlotKey(42))));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel::from(SlotKey(0))));

		app.update();

		assert_eq!(
			Some(&Dad::from(SlotKey(42))),
			app.world().entity(agent).get::<Dad>()
		);
	}

	#[test]
	fn no_drag_when_not_pressed() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Player).id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel::from(SlotKey(42))));

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<Dad>());
	}
}
