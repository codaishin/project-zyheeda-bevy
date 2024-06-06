use crate::components::Label;
use bevy::{
	ecs::{
		query::Added,
		system::{Query, Resource},
		world::Mut,
	},
	prelude::{default, Res},
	text::{Text, TextSection},
};
use common::traits::get_ui_text::{GetUiTextFor, UIText};
use skills::items::slot_key::SlotKey;

type Labels<'a, T> = (&'a Label<T, SlotKey>, &'a mut Text);

pub fn update_label_text<
	TLanguageServer: Resource + GetUiTextFor<SlotKey>,
	T: Sync + Send + 'static,
>(
	map: Res<TLanguageServer>,
	mut labels: Query<Labels<T>, Added<Label<T, SlotKey>>>,
) {
	for (label, text) in &mut labels {
		update_text(&map, label, text);
	}
}

fn update_text<TLanguageServer: Resource + GetUiTextFor<SlotKey>, T>(
	map: &Res<TLanguageServer>,
	label: &Label<T, SlotKey>,
	mut text: Mut<Text>,
) {
	let UIText::String(value) = map.ui_text_for(&label.key) else {
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

	struct _T;

	#[derive(Resource)]
	struct _LanguageServer(SlotKey, &'static str);

	impl GetUiTextFor<SlotKey> for _LanguageServer {
		fn ui_text_for(&self, value: &SlotKey) -> UIText {
			if value != &self.0 {
				return UIText::Unmapped;
			}
			UIText::from(self.1)
		}
	}

	#[test]
	fn add_section_to_text() {
		let mut app = App::new();

		app.insert_resource(_LanguageServer(SlotKey::Hand(Side::Main), "IIIIII"));
		let id = app
			.world
			.spawn((
				Label::<_T, SlotKey>::new(SlotKey::Hand(Side::Main)),
				Text::default(),
			))
			.id();

		app.add_systems(Update, update_label_text::<_LanguageServer, _T>);
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

		app.insert_resource(_LanguageServer(SlotKey::Hand(Side::Main), "IIIIII"));
		let id = app
			.world
			.spawn((
				Label::<_T, SlotKey>::new(SlotKey::Hand(Side::Main)),
				Text::from_section("OVERRIDE THIS", default()),
			))
			.id();

		app.add_systems(Update, update_label_text::<_LanguageServer, _T>);
		app.update();

		let text = app.world.entity(id).get::<Text>().unwrap();

		assert_eq!(
			Some("IIIIII".to_owned()),
			text.sections.first().map(|t| t.value.clone())
		)
	}
}
