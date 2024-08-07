use crate::components::{Dad, KeyedPanel};
use bevy::{
	ecs::{
		component::Component,
		query::With,
		system::{Commands, Query},
	},
	input::ButtonInput,
	prelude::{Entity, MouseButton, Res},
	ui::Interaction,
};
use common::components::{Collection, Swap};

pub fn drop<
	TAgent: Component,
	TKeyDad: Send + Sync + Copy + 'static,
	TKeyKeyedPanel: Send + Sync + Copy + 'static,
>(
	mut commands: Commands,
	agents: Query<(Entity, &Dad<TKeyDad>), With<TAgent>>,
	panels: Query<(&Interaction, &KeyedPanel<TKeyKeyedPanel>)>,
	mouse: Res<ButtonInput<MouseButton>>,
) {
	if !mouse.just_released(MouseButton::Left) {
		return;
	}

	let Ok((agent, dad)) = agents.get_single() else {
		return;
	};

	let Some((.., keyed_panel)) = panels.iter().find(is_hovered) else {
		return;
	};

	let Some(mut agent) = commands.get_entity(agent) else {
		return;
	};

	agent.try_insert(Collection::new([Swap(dad.0, keyed_panel.0)]));
	agent.remove::<Dad<TKeyDad>>();
}

fn is_hovered<TDadPanel>((interaction, _): &(&Interaction, &KeyedPanel<TDadPanel>)) -> bool {
	Interaction::Hovered == **interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ui::Interaction,
	};

	#[derive(Component)]
	struct _Agent;

	fn setup<T1: Copy + Send + Sync + 'static, T2: Copy + Send + Sync + 'static>() -> App {
		let mut app = App::new();
		app.insert_resource(ButtonInput::<MouseButton>::default());
		app.add_systems(Update, drop::<_Agent, T1, T2>);

		app
	}

	fn press_and_release_mouse_left(app: &mut App) {
		app.world_mut()
			.get_resource_mut::<ButtonInput<MouseButton>>()
			.unwrap()
			.press(MouseButton::Left);
		app.update();
		app.world_mut()
			.get_resource_mut::<ButtonInput<MouseButton>>()
			.unwrap()
			.release(MouseButton::Left);
	}

	#[test]
	fn add_result_component() {
		let mut app = setup::<usize, f32>();
		let agent = app.world_mut().spawn((_Agent, Dad(42_usize))).id();

		press_and_release_mouse_left(&mut app);
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Collection::new([Swap(42_usize, 11_f32)])),
			agent.get::<Collection<Swap<usize, f32>>>()
		);
	}

	#[test]
	fn add_result_component_when_multiple_panels_exist() {
		let mut app = setup::<usize, f32>();
		let agent = app.world_mut().spawn((_Agent, Dad(42_usize))).id();

		press_and_release_mouse_left(&mut app);
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(0_f32)));
		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Collection::new([Swap(42_usize, 11_f32)])),
			agent.get::<Collection<Swap<usize, f32>>>()
		);
	}

	#[test]
	fn no_panic_when_agent_has_no_dad() {
		let mut app = setup::<usize, f32>();
		app.world_mut().spawn(_Agent);

		press_and_release_mouse_left(&mut app);
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(0_f32)));
		app.update();
	}

	#[test]
	fn no_result_when_interaction_not_hover() {
		let mut app = setup::<usize, f32>();
		let agent = app.world_mut().spawn((_Agent, Dad(42_usize))).id();

		press_and_release_mouse_left(&mut app);
		app.world_mut()
			.spawn((Interaction::Pressed, KeyedPanel(11_f32)));
		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<Collection<Swap<usize, f32>>>());
	}

	#[test]
	fn no_result_when_not_mouse_left_release() {
		let mut app = setup::<usize, f32>();
		let agent = app.world_mut().spawn((_Agent, Dad(42_usize))).id();

		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<Collection<Swap<usize, f32>>>());
	}

	#[test]
	fn remove_dad() {
		let mut app = setup::<usize, f32>();
		let agent = app.world_mut().spawn((_Agent, Dad(42_usize))).id();

		press_and_release_mouse_left(&mut app);
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.update();

		let agent = app.world().entity(agent);

		assert!(!agent.contains::<Dad<usize>>());
	}
}
