use crate::components::{Dad, KeyedPanel};
use bevy::{
	ecs::{component::Component, query::With, system::Query},
	prelude::Entity,
	ui::Interaction,
};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

pub fn drag_item<TAgent>(
	mut commands: ZyheedaCommands,
	agents: Query<Entity, With<TAgent>>,
	panels: Query<(&Interaction, &KeyedPanel)>,
) where
	TAgent: Component,
{
	let Some((.., panel)) = panels.iter().find(is_pressed) else {
		return;
	};

	for entity in &agents {
		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(Dad(panel.0));
		});
	}
}

fn is_pressed((interaction, _): &(&Interaction, &KeyedPanel)) -> bool {
	Interaction::Pressed == **interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Dad;
	use bevy::app::{App, Update};
	use common::tools::action_key::slot::SlotKey;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, drag_item::<_Agent>);

		app
	}

	#[test]
	fn drag_panel_on_pressed() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();
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
		let agent = app.world_mut().spawn(_Agent).id();
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
		let agent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel::from(SlotKey(42))));

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<Dad>());
	}
}
