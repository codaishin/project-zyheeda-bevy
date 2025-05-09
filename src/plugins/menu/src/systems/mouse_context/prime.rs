use bevy::{
	ecs::{
		component::Component,
		system::{Query, Res, ResMut, Resource},
	},
	state::state::NextState,
	ui::Interaction,
};
use common::{
	states::mouse_context::MouseContext,
	tools::action_key::{slot::SlotKey, user_input::UserInput},
	traits::{accessors::get::GetterRef, key_mappings::GetInput},
};

pub fn prime_mouse_context<
	TMap: Resource + GetInput<SlotKey, UserInput>,
	TPanel: GetterRef<SlotKey> + Component,
>(
	mut mouse_context: ResMut<NextState<MouseContext>>,
	key_map: Res<TMap>,
	buttons: Query<(&TPanel, &Interaction)>,
) {
	let user_input = get_key_code_from(key_map.as_ref());
	let user_input = buttons.iter().find(is_pressed).map(user_input);

	let Some(user_input) = user_input else {
		return;
	};

	mouse_context.set(MouseContext::Primed(user_input));
}

fn get_key_code_from<TMap: GetInput<SlotKey, UserInput>, TPanel: GetterRef<SlotKey>>(
	key_map: &'_ TMap,
) -> impl Fn((&TPanel, &Interaction)) -> UserInput + '_ {
	|(panel, _): (&TPanel, &Interaction)| key_map.get_input(*panel.get())
}

fn is_pressed<TPanel>((_, interaction): &(&TPanel, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod test {
	use super::*;
	use bevy::{
		app::{App, Update},
		input::keyboard::KeyCode,
		state::{
			app::{AppExtStates, StatesPlugin},
			state::State,
		},
		ui::Interaction,
	};
	use common::tools::action_key::slot::Side;

	#[derive(Component)]
	struct _Panel(pub SlotKey);

	impl GetterRef<SlotKey> for _Panel {
		fn get(&self) -> &SlotKey {
			&self.0
		}
	}

	#[derive(Resource)]
	struct _Map(SlotKey, UserInput);

	impl GetInput<SlotKey, UserInput> for _Map {
		fn get_input(&self, value: SlotKey) -> UserInput {
			if value == self.0 {
				return self.1;
			}
			UserInput::from(KeyCode::Abort)
		}
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new();

		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.insert_resource(map);
		app.add_systems(Update, prime_mouse_context::<_Map, _Panel>);

		app
	}

	#[test]
	fn prime() {
		let mut app = setup(_Map(
			SlotKey::BottomHand(Side::Right),
			UserInput::from(KeyCode::KeyZ),
		));

		app.world_mut().spawn((
			_Panel(SlotKey::BottomHand(Side::Right)),
			Interaction::Pressed,
		));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		let mouse_context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.map(|s| s.get());

		assert_eq!(
			Some(&MouseContext::Primed(UserInput::from(KeyCode::KeyZ))),
			mouse_context
		);
	}

	#[test]
	fn do_not_prime_when_not_pressed() {
		let mut app = setup(_Map(
			SlotKey::BottomHand(Side::Right),
			UserInput::from(KeyCode::KeyZ),
		));

		app.world_mut()
			.spawn((_Panel(SlotKey::BottomHand(Side::Right)), Interaction::None));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		let mouse_context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.map(|s| s.get());

		assert_eq!(Some(&MouseContext::Default), mouse_context);
	}

	#[test]
	fn prime_with_different_key() {
		let mut app = setup(_Map(
			SlotKey::BottomHand(Side::Right),
			UserInput::from(KeyCode::KeyT),
		));

		app.world_mut().spawn((
			_Panel(SlotKey::BottomHand(Side::Right)),
			Interaction::Pressed,
		));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		let mouse_context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.map(|s| s.get());

		assert_eq!(
			Some(&MouseContext::Primed(UserInput::from(KeyCode::KeyT))),
			mouse_context
		);
	}
}
