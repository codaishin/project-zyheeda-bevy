use crate::components::Label;
use bevy::{
	ecs::{
		query::Added,
		system::{Query, Resource},
		world::Mut,
	},
	input::keyboard::KeyCode,
	prelude::{default, Res},
	text::{Text, TextSection},
};
use common::traits::{
	get_ui_text::{GetUiTextFor, UIText},
	map_value::MapForward,
};
use skills::items::slot_key::SlotKey;

type Labels<'a, T> = (&'a Label<T, SlotKey>, &'a mut Text);

pub fn update_label_text<
	TMap: Resource + MapForward<SlotKey, KeyCode>,
	TLanguageServer: Resource + GetUiTextFor<KeyCode>,
	T: Sync + Send + 'static,
>(
	key_map: Res<TMap>,
	language_server: Res<TLanguageServer>,
	mut labels: Query<Labels<T>, Added<Label<T, SlotKey>>>,
) {
	let key_map = key_map.as_ref();
	let language_server = language_server.as_ref();

	for (label, text) in &mut labels {
		update_text(key_map, language_server, label, text);
	}
}

fn update_text<TMap: MapForward<SlotKey, KeyCode>, TLanguageServer: GetUiTextFor<KeyCode>, T>(
	key_map: &TMap,
	language_server: &TLanguageServer,
	label: &Label<T, SlotKey>,
	mut text: Mut<Text>,
) {
	let key = key_map.map_forward(label.key);
	let UIText::String(value) = language_server.ui_text_for(&key) else {
		return;
	};
	let update = match text.sections.is_empty() {
		true => add_first_section,
		false => set_first_section,
	};
	update(&mut text, &value);
}

fn add_first_section(text: &mut Mut<Text>, value: &str) {
	text.sections.push(TextSection {
		value: value.to_string(),
		..default()
	});
}

fn set_first_section(text: &mut Mut<Text>, value: &str) {
	text.sections[0].value = value.to_string()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use common::components::Side;
	use mockall::{automock, predicate::eq};

	struct _T;

	#[derive(Resource, Default)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl MapForward<SlotKey, KeyCode> for _Map {
		fn map_forward(&self, value: SlotKey) -> KeyCode {
			self.mock.map_forward(value)
		}
	}

	#[derive(Resource)]
	struct _LanguageServer(KeyCode, &'static str);

	impl GetUiTextFor<KeyCode> for _LanguageServer {
		fn ui_text_for(&self, value: &KeyCode) -> UIText {
			if value != &self.0 {
				return UIText::Unmapped;
			}
			UIText::from(self.1)
		}
	}

	#[test]
	fn add_section_to_text() {
		let mut app = App::new();
		let mut map = _Map::default();
		map.mock.expect_map_forward().return_const(KeyCode::ArrowUp);

		app.insert_resource(map);
		app.insert_resource(_LanguageServer(KeyCode::ArrowUp, "IIIIII"));
		let id = app
			.world
			.spawn((
				Label::<_T, SlotKey>::new(SlotKey::Hand(Side::Main)),
				Text::default(),
			))
			.id();

		app.add_systems(Update, update_label_text::<_Map, _LanguageServer, _T>);
		app.update();

		let text = app.world.entity(id).get::<Text>().unwrap();

		assert_eq!(
			Some("IIIIII".to_owned()),
			text.sections.first().map(|t| t.value.clone())
		)
	}

	#[test]
	fn override_first_section() {
		let mut app = App::new();
		let mut map = _Map::default();
		map.mock.expect_map_forward().return_const(KeyCode::ArrowUp);

		app.insert_resource(map);
		app.insert_resource(_LanguageServer(KeyCode::ArrowUp, "IIIIII"));
		let id = app
			.world
			.spawn((
				Label::<_T, SlotKey>::new(SlotKey::Hand(Side::Main)),
				Text::from_section("OVERRIDE THIS", default()),
			))
			.id();

		app.add_systems(Update, update_label_text::<_Map, _LanguageServer, _T>);
		app.update();

		let text = app.world.entity(id).get::<Text>().unwrap();

		assert_eq!(
			Some("IIIIII".to_owned()),
			text.sections.first().map(|t| t.value.clone())
		)
	}

	#[test]
	fn map_slot_key_properly() {
		let mut app = App::new();
		let mut map = _Map::default();
		map.mock
			.expect_map_forward()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Off)))
			.return_const(KeyCode::ArrowUp);

		app.insert_resource(map);
		app.insert_resource(_LanguageServer(KeyCode::ArrowUp, "IIIIII"));
		app.world.spawn((
			Label::<_T, SlotKey>::new(SlotKey::Hand(Side::Off)),
			Text::from_section("OVERRIDE THIS", default()),
		));

		app.add_systems(Update, update_label_text::<_Map, _LanguageServer, _T>);
		app.update();
	}
}
