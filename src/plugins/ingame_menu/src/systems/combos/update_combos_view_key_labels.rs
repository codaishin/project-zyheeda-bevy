use crate::components::key_select::KeySelect;
use bevy::{
	prelude::{KeyCode, Mut, Parent, Query, Res, Resource},
	text::Text,
	ui::Interaction,
};
use common::traits::get_ui_text::GetUiTextFor;

pub(crate) fn update_combos_view_key_labels<TLanguageServer: GetUiTextFor<KeyCode> + Resource>(
	key_selects: Query<(&KeySelect, &Interaction)>,
	language_server: Res<TLanguageServer>,
	mut texts: Query<(&mut Text, &Parent)>,
) {
	for (key_select, ..) in key_selects.iter().filter(pressed) {
		set_key_label(&mut texts, key_select, &language_server);
	}
}

fn pressed((.., interaction): &(&KeySelect, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

fn set_key_label<TLanguageServer: GetUiTextFor<KeyCode> + Resource>(
	texts: &mut Query<(&mut Text, &Parent)>,
	key_select: &KeySelect,
	language_server: &Res<TLanguageServer>,
) -> Option<()> {
	let (mut text, ..) = get_text(texts, key_select)?;
	let section = text.sections.get_mut(0)?;
	let key_code = key_select.key_path.last()?;
	let key_text = language_server.ui_text_for(key_code).ok()?;

	section.value = key_text;

	Some(())
}

fn get_text<'a>(
	texts: &'a mut Query<(&mut Text, &Parent)>,
	key_select: &'a KeySelect,
) -> Option<(Mut<'a, Text>, &'a Parent)> {
	texts
		.iter_mut()
		.find(|(_, parent)| parent.get() == key_select.key_button)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::{BuildWorldChildren, Entity, KeyCode, Resource, TextBundle},
		text::Text,
		utils::default,
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::get_ui_text::UIText};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Default)]
	struct _LanguageServer {
		mock: Mock_LanguageServer,
	}

	#[automock]
	impl GetUiTextFor<KeyCode> for _LanguageServer {
		fn ui_text_for(&self, value: &KeyCode) -> UIText {
			self.mock.ui_text_for(value)
		}
	}

	fn setup(language_server: _LanguageServer) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(language_server);
		app.add_systems(Update, update_combos_view_key_labels::<_LanguageServer>);

		app
	}

	#[test]
	fn set_key_button_target() {
		let mut language_server = _LanguageServer::default();
		language_server
			.mock
			.expect_ui_text_for()
			.return_const(UIText::from("key text"));

		let mut app = setup(language_server);
		let key_button = app.world_mut().spawn_empty().id();
		let text = app
			.world_mut()
			.spawn(TextBundle::from_section("", default()))
			.set_parent(key_button)
			.id();
		app.world_mut().spawn((
			Interaction::Pressed,
			KeySelect {
				skill_button: Entity::from_raw(101),
				key_button,
				key_path: vec![KeyCode::KeyB],
			},
		));

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();

		assert_eq!("key text", text.sections[0].value)
	}

	#[test]
	fn call_language_server_with_proper_key_code() {
		let mut language_server = _LanguageServer::default();
		language_server
			.mock
			.expect_ui_text_for()
			.times(1)
			.with(eq(KeyCode::KeyE))
			.return_const(UIText::Unmapped);

		let mut app = setup(language_server);
		let key_button = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn(TextBundle::from_section("", default()))
			.set_parent(key_button);
		app.world_mut().spawn((
			Interaction::Pressed,
			KeySelect {
				skill_button: Entity::from_raw(101),
				key_button,
				key_path: vec![KeyCode::KeyZ, KeyCode::KeyQ, KeyCode::KeyA, KeyCode::KeyE],
			},
		));

		app.update();
	}

	#[test]
	fn do_nothing_when_interaction_not_pressed() {
		let mut language_server = _LanguageServer::default();
		language_server
			.mock
			.expect_ui_text_for()
			.never()
			.return_const(UIText::from("key text"));

		let mut app = setup(language_server);
		let key_button = app.world_mut().spawn_empty().id();
		let text = app
			.world_mut()
			.spawn(TextBundle::from_section("unchanged", default()))
			.set_parent(key_button)
			.id();
		app.world_mut().spawn((
			Interaction::Hovered,
			KeySelect {
				skill_button: Entity::from_raw(101),
				key_button,
				key_path: vec![KeyCode::KeyB],
			},
		));
		app.world_mut().spawn((
			Interaction::None,
			KeySelect {
				skill_button: Entity::from_raw(101),
				key_button,
				key_path: vec![KeyCode::KeyB],
			},
		));

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();

		assert_eq!("unchanged", text.sections[0].value)
	}
}
