use crate::components::{Collection, Dad, KeyedPanel};
use bevy::{
	ecs::{
		component::Component,
		query::With,
		system::{Commands, Query},
	},
	prelude::{Entity, Input, MouseButton, Res},
	ui::Interaction,
};

pub fn drop<
	TAgent: Component,
	TKeyDad: Send + Sync + Copy + 'static,
	TKeyKeyedPanel: Send + Sync + Copy + 'static,
	TResult: From<(Dad<TKeyDad>, KeyedPanel<TKeyKeyedPanel>)> + Send + Sync + 'static,
>(
	mut commands: Commands,
	agents: Query<(Entity, &Dad<TKeyDad>), With<TAgent>>,
	panels: Query<(&Interaction, &KeyedPanel<TKeyKeyedPanel>)>,
	mouse: Res<Input<MouseButton>>,
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

	let mut agent = commands.entity(agent);
	agent.insert(Collection::new([TResult::from((*dad, *keyed_panel))]));
	agent.remove::<Dad<TKeyDad>>();
}

fn is_hovered<TDadPanel>((interaction, _): &(&Interaction, &KeyedPanel<TDadPanel>)) -> bool {
	Interaction::Hovered == **interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Collection;
	use bevy::{
		app::{App, Update},
		ui::Interaction,
	};

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq)]
	struct _Result<T1, T2>(T1, T2);

	impl<T1, T2> From<(Dad<T1>, KeyedPanel<T2>)> for _Result<T1, T2> {
		fn from(value: (Dad<T1>, KeyedPanel<T2>)) -> Self {
			let (dad, keyed_panel) = value;
			Self(dad.0, keyed_panel.0)
		}
	}

	fn setup<T1: Copy + Send + Sync + 'static, T2: Copy + Send + Sync + 'static>() -> App {
		let mut app = App::new();
		app.insert_resource(Input::<MouseButton>::default());
		app.add_systems(Update, drop::<_Agent, T1, T2, _Result<T1, T2>>);

		app
	}

	fn press_and_release_mouse_left(app: &mut App) {
		app.world
			.get_resource_mut::<Input<MouseButton>>()
			.unwrap()
			.press(MouseButton::Left);
		app.update();
		app.world
			.get_resource_mut::<Input<MouseButton>>()
			.unwrap()
			.release(MouseButton::Left);
	}

	#[test]
	fn add_result_component() {
		let mut app = setup::<usize, f32>();
		let agent = app.world.spawn((_Agent, Dad(42_usize))).id();

		press_and_release_mouse_left(&mut app);
		app.world.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Collection::new([_Result(42_usize, 11_f32)])),
			agent.get::<Collection<_Result<usize, f32>>>()
		);
	}

	#[test]
	fn add_result_component_when_multiple_panels_exist() {
		let mut app = setup::<usize, f32>();
		let agent = app.world.spawn((_Agent, Dad(42_usize))).id();

		press_and_release_mouse_left(&mut app);
		app.world.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.world.spawn((Interaction::None, KeyedPanel(0_f32)));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Collection::new([_Result(42_usize, 11_f32)])),
			agent.get::<Collection<_Result<usize, f32>>>()
		);
	}

	#[test]
	fn no_panic_when_agent_has_no_dad() {
		let mut app = setup::<usize, f32>();
		app.world.spawn(_Agent);

		press_and_release_mouse_left(&mut app);
		app.world.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.world.spawn((Interaction::None, KeyedPanel(0_f32)));
		app.update();
	}

	#[test]
	fn no_result_when_interaction_not_hover() {
		let mut app = setup::<usize, f32>();
		let agent = app.world.spawn((_Agent, Dad(42_usize))).id();

		press_and_release_mouse_left(&mut app);
		app.world.spawn((Interaction::Pressed, KeyedPanel(11_f32)));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Collection<_Result<usize, f32>>>());
	}

	#[test]
	fn no_result_when_not_mouse_left_release() {
		let mut app = setup::<usize, f32>();
		let agent = app.world.spawn((_Agent, Dad(42_usize))).id();

		app.world.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Collection<_Result<usize, f32>>>());
	}

	#[test]
	fn remove_dad() {
		let mut app = setup::<usize, f32>();
		let agent = app.world.spawn((_Agent, Dad(42_usize))).id();

		press_and_release_mouse_left(&mut app);
		app.world.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Dad<usize>>());
	}
}
