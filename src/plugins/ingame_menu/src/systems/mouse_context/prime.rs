use bevy::{
	ecs::{
		component::Component,
		schedule::NextState,
		system::{Query, Res, ResMut},
	},
	input::keyboard::KeyCode,
	ui::Interaction,
};
use common::{states::MouseContext, traits::get::GetStatic};
use skills::{items::SlotKey, resources::SlotMap};

pub fn prime_mouse_context<TPanel: GetStatic<SlotKey> + Component>(
	mut mouse_context: ResMut<NextState<MouseContext>>,
	slot_map: Res<SlotMap<KeyCode>>,
	buttons: Query<(&TPanel, &Interaction)>,
) {
	let get_key_code = get_key_code_from(&slot_map);
	let key_code = buttons.iter().filter(is_pressed).find_map(get_key_code);

	let Some(key_code) = key_code else {
		return;
	};
	mouse_context.set(MouseContext::Primed(*key_code));
}

fn get_key_code_from<'a, TPanel: GetStatic<SlotKey>>(
	slot_map: &'a SlotMap<KeyCode>,
) -> impl Fn((&TPanel, &Interaction)) -> Option<&'a KeyCode> {
	|(panel, _): (&TPanel, &Interaction)| slot_map.keys.get(panel.get())
}

fn is_pressed<TPanel>((_, interaction): &(&TPanel, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod test {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::schedule::{NextState, State},
		input::keyboard::KeyCode,
		ui::Interaction,
	};
	use common::components::Side;

	#[derive(Component)]
	struct _Panel(pub SlotKey);

	impl GetStatic<SlotKey> for _Panel {
		fn get(&self) -> &SlotKey {
			&self.0
		}
	}

	#[test]
	fn prime() {
		let mut app = App::new();

		app.init_state::<MouseContext>();
		app.insert_resource(SlotMap::new([(
			KeyCode::KeyZ,
			SlotKey::Hand(Side::Main),
			"",
		)]));
		app.world
			.spawn((_Panel(SlotKey::Hand(Side::Main)), Interaction::Pressed));
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.add_systems(Update, prime_mouse_context::<_Panel>);
		app.update();
		app.update();

		let mouse_context = app
			.world
			.get_resource::<State<MouseContext>>()
			.map(|s| s.get());

		assert_eq!(Some(&MouseContext::Primed(KeyCode::KeyZ)), mouse_context);
	}

	#[test]
	fn do_not_prime_when_not_pressed() {
		let mut app = App::new();

		app.init_state::<MouseContext>();
		app.insert_resource(SlotMap::new([(
			KeyCode::KeyZ,
			SlotKey::Hand(Side::Main),
			"",
		)]));
		app.world
			.spawn((_Panel(SlotKey::Hand(Side::Main)), Interaction::None));
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.add_systems(Update, prime_mouse_context::<_Panel>);
		app.update();
		app.update();

		let mouse_context = app
			.world
			.get_resource::<State<MouseContext>>()
			.map(|s| s.get());

		assert_eq!(Some(&MouseContext::Default), mouse_context);
	}

	#[test]
	fn prime_with_different_key() {
		let mut app = App::new();

		app.init_state::<MouseContext>();
		app.insert_resource(SlotMap::new([(
			KeyCode::KeyT,
			SlotKey::Hand(Side::Main),
			"",
		)]));
		app.world
			.spawn((_Panel(SlotKey::Hand(Side::Main)), Interaction::Pressed));
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.add_systems(Update, prime_mouse_context::<_Panel>);
		app.update();
		app.update();

		let mouse_context = app
			.world
			.get_resource::<State<MouseContext>>()
			.map(|s| s.get());

		assert_eq!(Some(&MouseContext::Primed(KeyCode::KeyT)), mouse_context);
	}
}
