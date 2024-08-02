use crate::components::key_select::KeySelect;
use bevy::{
	prelude::{Mut, Parent, Query, Res, Resource},
	text::Text,
	ui::Interaction,
};
use common::traits::get_ui_text::GetUiTextFor;
use skills::items::slot_key::SlotKey;

pub(crate) fn update_combos_view_key_labels<TLanguageServer, TExtra>(
	key_selects: Query<(&KeySelect<TExtra>, &Interaction)>,
	language_server: Res<TLanguageServer>,
	mut texts: Query<(&mut Text, &Parent)>,
) where
	TLanguageServer: GetUiTextFor<SlotKey> + Resource,
	TExtra: Clone + Sync + Send + 'static,
{
	for (key_select, ..) in key_selects.iter().filter(pressed) {
		set_key_label(&mut texts, key_select, &language_server);
	}
}

fn pressed<TExtra>((.., interaction): &(&KeySelect<TExtra>, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

fn set_key_label<TLanguageServer: GetUiTextFor<SlotKey> + Resource, TExtra>(
	texts: &mut Query<(&mut Text, &Parent)>,
	key_select: &KeySelect<TExtra>,
	language_server: &Res<TLanguageServer>,
) -> Option<()> {
	let (mut text, ..) = get_text(texts, key_select)?;
	let section = text.sections.get_mut(0)?;
	let slot_key = key_select.key_path.last()?;
	let key_text = language_server.ui_text_for(slot_key).ok()?;

	section.value = key_text;

	Some(())
}

fn get_text<'a, TExtra>(
	texts: &'a mut Query<(&mut Text, &Parent)>,
	key_select: &'a KeySelect<TExtra>,
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
		prelude::{BuildWorldChildren, KeyCode, Resource, TextBundle},
		text::Text,
		utils::default,
	};
	use common::{
		components::Side,
		test_tools::utils::SingleThreadedApp,
		traits::get_ui_text::UIText,
	};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Default)]
	struct _LanguageServer {
		mock: Mock_LanguageServer,
	}

	#[automock]
	impl GetUiTextFor<SlotKey> for _LanguageServer {
		fn ui_text_for(&self, value: &SlotKey) -> UIText {
			self.mock.ui_text_for(value)
		}
	}

	fn setup(language_server: _LanguageServer) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(language_server);
		app.add_systems(Update, update_combos_view_key_labels::<_LanguageServer, ()>);

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
				extra: (),
				key_button,
				key_path: vec![SlotKey::Hand(Side::Main)],
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
			.with(eq(SlotKey::Hand(Side::Main)))
			.return_const(UIText::Unmapped);

		let mut app = setup(language_server);
		let key_button = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn(TextBundle::from_section("", default()))
			.set_parent(key_button);
		app.world_mut().spawn((
			Interaction::Pressed,
			KeySelect {
				extra: (),
				key_button,
				key_path: vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
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
				extra: (),
				key_button,
				key_path: vec![KeyCode::KeyB],
			},
		));
		app.world_mut().spawn((
			Interaction::None,
			KeySelect {
				extra: (),
				key_button,
				key_path: vec![KeyCode::KeyB],
			},
		));

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();

		assert_eq!("unchanged", text.sections[0].value)
	}
}
