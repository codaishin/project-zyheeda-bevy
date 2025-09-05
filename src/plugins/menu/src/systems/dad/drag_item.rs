use crate::components::{Dad, KeyedPanel};
use bevy::{
	ecs::{component::Component, query::With, system::Query},
	prelude::Entity,
	ui::Interaction,
};
use common::{
	traits::{accessors::get::TryApplyOn, handles_loadout::loadout::LoadoutKey},
	zyheeda_commands::ZyheedaCommands,
};

pub fn drag_item<TAgent, TContainer>(
	mut commands: ZyheedaCommands,
	agents: Query<Entity, With<TAgent>>,
	panels: Query<(&Interaction, &KeyedPanel<TContainer::TKey>)>,
) where
	TAgent: Component,
	TContainer: LoadoutKey,
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

fn is_pressed<TKeyedPanel>((interaction, _): &(&Interaction, &KeyedPanel<TKeyedPanel>)) -> bool {
	Interaction::Pressed == **interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Dad;
	use bevy::app::{App, Update};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	struct _Container;

	impl LoadoutKey for _Container {
		type TKey = u32;
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, drag_item::<_Agent, _Container>);

		app
	}

	#[test]
	fn drag_panel_on_pressed() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((Interaction::Pressed, KeyedPanel(42_u32)));

		app.update();

		assert_eq!(Some(&Dad(42)), app.world().entity(agent).get::<Dad<u32>>());
	}

	#[test]
	fn drag_panel_on_pressed_when_multiple_panels_exist() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((Interaction::Pressed, KeyedPanel(42_u32)));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(0_u32)));

		app.update();

		assert_eq!(Some(&Dad(42)), app.world().entity(agent).get::<Dad<u32>>());
	}

	#[test]
	fn no_drag_when_not_pressed() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(42_u32)));

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<Dad<u32>>());
	}
}
