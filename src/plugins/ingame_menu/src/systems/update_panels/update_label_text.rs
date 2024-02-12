use crate::components::Label;
use bevy::{
	ecs::{query::Added, system::Query, world::Mut},
	input::keyboard::KeyCode,
	prelude::{default, Res},
	text::{Text, TextSection},
};
use common::{components::SlotKey, resources::SlotMap};

type Labels<'a, T> = (&'a Label<T, SlotKey>, &'a mut Text);

pub fn update_label_text<T: Sync + Send + 'static>(
	map: Res<SlotMap<KeyCode>>,
	mut labels: Query<Labels<T>, Added<Label<T, SlotKey>>>,
) {
	for (label, text) in &mut labels {
		update_text(&map, label, text);
	}
}

fn update_text<T>(map: &Res<SlotMap<KeyCode>>, label: &Label<T, SlotKey>, mut text: Mut<Text>) {
	let Some(value) = map.ui_input_display.get(&label.key) else {
		return;
	};
	let update = match text.sections.is_empty() {
		true => add_first_section,
		false => set_first_section,
	};
	update(&mut text, value);
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

	struct _T;

	#[test]
	fn add_section_to_text() {
		let mut app = App::new();

		app.insert_resource(SlotMap::new([(KeyCode::Q, SlotKey::Legs, "IIIIII")]));
		let id = app
			.world
			.spawn((Label::<_T, SlotKey>::new(SlotKey::Legs), Text::default()))
			.id();

		app.add_systems(Update, update_label_text::<_T>);
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

		app.insert_resource(SlotMap::new([(KeyCode::Q, SlotKey::Legs, "IIIIII")]));
		let id = app
			.world
			.spawn((
				Label::<_T, SlotKey>::new(SlotKey::Legs),
				Text::from_section("OVERRIDE THIS", default()),
			))
			.id();

		app.add_systems(Update, update_label_text::<_T>);
		app.update();

		let text = app.world.entity(id).get::<Text>().unwrap();

		assert_eq!(
			Some("IIIIII".to_owned()),
			text.sections.first().map(|t| t.value.clone())
		)
	}
}
