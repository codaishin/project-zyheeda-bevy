use bevy::{
	ecs::{
		component::Component,
		system::{Query, Res, ResMut, Resource},
	},
	input::keyboard::KeyCode,
	state::state::NextState,
	ui::Interaction,
};
use common::{
	states::mouse_context::MouseContext,
	tools::slot_key::SlotKey,
	traits::{accessors::get::GetterRef, key_mappings::GetKeyCode},
};

pub fn prime_mouse_context<
	TMap: Resource + GetKeyCode<SlotKey, KeyCode>,
	TPanel: GetterRef<SlotKey> + Component,
>(
	mut mouse_context: ResMut<NextState<MouseContext>>,
	key_map: Res<TMap>,
	buttons: Query<(&TPanel, &Interaction)>,
) {
	let get_key_code = get_key_code_from(key_map.as_ref());
	let key_code = buttons.iter().find(is_pressed).map(get_key_code);

	let Some(key_code) = key_code else {
		return;
	};

	mouse_context.set(MouseContext::Primed(key_code));
}

fn get_key_code_from<TMap: GetKeyCode<SlotKey, KeyCode>, TPanel: GetterRef<SlotKey>>(
	key_map: &'_ TMap,
) -> impl Fn((&TPanel, &Interaction)) -> KeyCode + '_ {
	|(panel, _): (&TPanel, &Interaction)| key_map.get_key_code(*panel.get())
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
	use common::tools::slot_key::Side;

	#[derive(Component)]
	struct _Panel(pub SlotKey);

	impl GetterRef<SlotKey> for _Panel {
		fn get(&self) -> &SlotKey {
			&self.0
		}
	}

	#[derive(Resource)]
	struct _Map(SlotKey, KeyCode);

	impl GetKeyCode<SlotKey, KeyCode> for _Map {
		fn get_key_code(&self, value: SlotKey) -> KeyCode {
			if value == self.0 {
				return self.1;
			}
			KeyCode::Abort
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
		let mut app = setup(_Map(SlotKey::BottomHand(Side::Right), KeyCode::KeyZ));

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

		assert_eq!(Some(&MouseContext::Primed(KeyCode::KeyZ)), mouse_context);
	}

	#[test]
	fn do_not_prime_when_not_pressed() {
		let mut app = setup(_Map(SlotKey::BottomHand(Side::Right), KeyCode::KeyZ));

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
		let mut app = setup(_Map(SlotKey::BottomHand(Side::Right), KeyCode::KeyT));

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

		assert_eq!(Some(&MouseContext::Primed(KeyCode::KeyT)), mouse_context);
	}
}
