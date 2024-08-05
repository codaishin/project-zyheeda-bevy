use crate::components::key_select::KeySelect;
use bevy::{
	prelude::{Mut, Parent, Query, Res, Resource},
	text::Text,
	ui::Interaction,
};
use common::traits::{get_ui_text::GetUiTextFor, map_value::MapForward};

pub(crate) fn update_combos_view_key_labels<TEquipmentKey, TKey, TMap, TLanguageServer, TExtra>(
	key_selects: Query<(&KeySelect<TExtra, TEquipmentKey>, &Interaction)>,
	map: Res<TMap>,
	language_server: Res<TLanguageServer>,
	mut texts: Query<(&mut Text, &Parent)>,
) where
	TEquipmentKey: Copy + Sync + Send + 'static,
	TKey: Copy + Sync + Send + 'static,
	TMap: MapForward<TEquipmentKey, TKey> + Resource,
	TLanguageServer: GetUiTextFor<TKey> + Resource,
	TExtra: Clone + Sync + Send + 'static,
{
	let map = map.as_ref();
	let language_server = language_server.as_ref();

	for (key_select, ..) in key_selects.iter().filter(pressed) {
		set_key_label(&mut texts, key_select, map, language_server);
	}
}

fn pressed<T>((.., interaction): &(&T, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

fn set_key_label<TEquipmentKey, TKey, TMap, TLanguageServer, TExtra>(
	texts: &mut Query<(&mut Text, &Parent)>,
	key_select: &KeySelect<TExtra, TEquipmentKey>,
	map: &TMap,
	language_server: &TLanguageServer,
) -> Option<()>
where
	TEquipmentKey: Copy,
	TMap: MapForward<TEquipmentKey, TKey>,
	TLanguageServer: GetUiTextFor<TKey>,
{
	let (mut text, ..) = get_text(texts, key_select)?;
	let section = text.sections.get_mut(0)?;
	let slot_key = key_select.key_path.last()?;
	let key = map.map_forward(*slot_key);
	let key_text = language_server.ui_text_for(&key).ok()?;

	section.value = key_text;

	Some(())
}

fn get_text<'a, TExtra, TEquipmentKey>(
	texts: &'a mut Query<(&mut Text, &Parent)>,
	key_select: &'a KeySelect<TExtra, TEquipmentKey>,
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
		test_tools::utils::SingleThreadedApp,
		traits::{get_ui_text::UIText, nested_mock::NestedMock},
	};
	use macros::NestedMock;
	use mockall::{automock, predicate::eq};

	#[derive(Clone, Copy, Debug, PartialEq)]
	struct _EquipmentKey(usize);

	#[derive(Clone, Copy, Debug, PartialEq)]
	struct _Key(usize);

	#[derive(Resource, NestedMock)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl MapForward<_EquipmentKey, _Key> for _Map {
		fn map_forward(&self, value: _EquipmentKey) -> _Key {
			self.mock.map_forward(value)
		}
	}

	#[derive(Resource, NestedMock)]
	struct _LanguageServer {
		mock: Mock_LanguageServer,
	}

	#[automock]
	impl GetUiTextFor<_Key> for _LanguageServer {
		fn ui_text_for(&self, value: &_Key) -> UIText {
			self.mock.ui_text_for(value)
		}
	}

	struct Setup {
		map: _Map,
		language: _LanguageServer,
	}

	impl Default for Setup {
		fn default() -> Self {
			Self {
				map: _Map::new_mock(|mock| {
					mock.expect_map_forward().return_const(_Key(0));
				}),
				language: _LanguageServer::new_mock(|mock| {
					mock.expect_ui_text_for()
						.return_const(UIText::String("".to_owned()));
				}),
			}
		}
	}

	fn setup(Setup { map, language }: Setup) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(map);
		app.insert_resource(language);
		app.add_systems(
			Update,
			update_combos_view_key_labels::<_EquipmentKey, _Key, _Map, _LanguageServer, ()>,
		);

		app
	}

	#[test]
	fn set_key_button_target() {
		let mut app = setup(Setup {
			language: _LanguageServer::new_mock(|mock| {
				mock.expect_ui_text_for()
					.return_const(UIText::from("key text"));
			}),
			..default()
		});
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
				key_path: vec![_EquipmentKey(0)],
			},
		));

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();

		assert_eq!("key text", text.sections[0].value)
	}

	#[test]
	fn map_the_last_key_of_the_key_path() {
		let mut app = setup(Setup {
			map: _Map::new_mock(|mock| {
				mock.expect_map_forward()
					.times(1)
					.with(eq(_EquipmentKey(123)))
					.return_const(_Key(123));
			}),
			language: _LanguageServer::new_mock(|mock| {
				mock.expect_ui_text_for()
					.times(1)
					.with(eq(_Key(123)))
					.return_const(UIText::Unmapped);
			}),
		});
		let key_button = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn(TextBundle::from_section("", default()))
			.set_parent(key_button);
		app.world_mut().spawn((
			Interaction::Pressed,
			KeySelect {
				extra: (),
				key_button,
				key_path: vec![_EquipmentKey(1), _EquipmentKey(12), _EquipmentKey(123)],
			},
		));

		app.update();
	}

	#[test]
	fn do_nothing_when_interaction_not_pressed() {
		let mut app = setup(Setup {
			language: _LanguageServer::new_mock(|mock| {
				mock.expect_ui_text_for()
					.never()
					.return_const(UIText::from("key text"));
			}),
			..default()
		});
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
