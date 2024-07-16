use crate::components::{Dad, KeyedPanel};
use bevy::{
	ecs::{
		component::Component,
		query::With,
		system::{Commands, Query},
	},
	prelude::Entity,
	ui::Interaction,
};
use common::traits::try_insert_on::TryInsertOn;

pub fn drag<TAgent: Component, TKey: Send + Sync + Copy + 'static>(
	mut commands: Commands,
	agents: Query<Entity, With<TAgent>>,
	panels: Query<(&Interaction, &KeyedPanel<TKey>)>,
) {
	let Some((.., panel)) = panels.iter().find(is_pressed) else {
		return;
	};

	let agent = agents.single();
	commands.try_insert_on(agent, Dad(panel.0));
}

fn is_pressed<TKeyedPanel>((interaction, _): &(&Interaction, &KeyedPanel<TKeyedPanel>)) -> bool {
	Interaction::Pressed == **interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Dad;
	use bevy::app::{App, Update};

	#[derive(Component)]
	struct _Agent;

	#[test]
	fn drag_panel_on_pressed() {
		let mut app = App::new();

		let agent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((Interaction::Pressed, KeyedPanel(42_u32)));
		app.add_systems(Update, drag::<_Agent, u32>);
		app.update();

		let agent = app.world().entity(agent);
		let dad = agent.get::<Dad<u32>>();

		assert_eq!(Some(&Dad(42)), dad);
	}

	#[test]
	fn drag_panel_on_pressed_when_multiple_panels_exist() {
		let mut app = App::new();

		let agent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((Interaction::Pressed, KeyedPanel(42_u32)));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(0_u32)));
		app.add_systems(Update, drag::<_Agent, u32>);
		app.update();

		let agent = app.world().entity(agent);
		let dad = agent.get::<Dad<u32>>();

		assert_eq!(Some(&Dad(42)), dad);
	}

	#[test]
	fn no_drag_when_not_pressed() {
		let mut app = App::new();

		let agent = app.world_mut().spawn(_Agent).id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(42_u32)));
		app.add_systems(Update, drag::<_Agent, u32>);
		app.update();

		let agent = app.world().entity(agent);
		let dad = agent.get::<Dad<u32>>();

		assert_eq!(None, dad);
	}
}
