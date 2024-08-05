use crate::components::key_code_text_insert_command::KeyCodeTextInsertCommand;
use bevy::prelude::{Commands, Entity, KeyCode, Query, Res, Resource, TextBundle};
use common::traits::{
	get_ui_text::{GetUiTextFor, UIText},
	map_value::MapForward,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};

pub(crate) fn insert_key_code_text<TKey, TKeyMap, TLanguageServer>(
	mut commands: Commands,
	insert_commands: Query<(Entity, &KeyCodeTextInsertCommand<TKey>)>,
	key_map: Res<TKeyMap>,
	language_server: Res<TLanguageServer>,
) where
	TKey: Copy + Sync + Send + 'static,
	TKeyMap: Resource + MapForward<TKey, KeyCode>,
	TLanguageServer: Resource + GetUiTextFor<KeyCode>,
{
	for (entity, insert_command) in &insert_commands {
		let key = key_map.map_forward(*insert_command.key());
		if let UIText::String(text) = language_server.ui_text_for(&key) {
			commands.try_insert_on(
				entity,
				TextBundle::from_section(text, insert_command.text_style().clone()),
			);
		};
		commands.try_remove_from::<KeyCodeTextInsertCommand<TKey>>(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		color::Color,
		prelude::TextBundle,
		text::{Text, TextStyle},
		utils::default,
	};
	use common::{
		assert_bundle,
		test_tools::utils::SingleThreadedApp,
		traits::{get_ui_text::UIText, nested_mock::NestedMock},
	};
	use macros::NestedMock;
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

	#[derive(Clone, Copy, Debug, PartialEq)]
	struct _Key;

	#[derive(Resource, NestedMock)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl MapForward<_Key, KeyCode> for _Map {
		fn map_forward(&self, value: _Key) -> KeyCode {
			self.mock.map_forward(value)
		}
	}

	#[derive(Resource, NestedMock)]
	struct _Language {
		mock: Mock_Language,
	}

	#[automock]
	impl GetUiTextFor<KeyCode> for _Language {
		fn ui_text_for(&self, value: &KeyCode) -> UIText {
			self.mock.ui_text_for(value)
		}
	}

	struct Setup {
		map: _Map,
		language: _Language,
	}

	impl Default for Setup {
		fn default() -> Self {
			Self {
				map: _Map::new_mock(|mock| {
					mock.expect_map_forward().return_const(KeyCode::KeyA);
				}),
				language: _Language::new_mock(|mock| {
					mock.expect_ui_text_for().return_const(UIText::Unmapped);
				}),
			}
		}
	}

	fn setup(Setup { map, language }: Setup) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(map);
		app.insert_resource(language);
		app.add_systems(Update, insert_key_code_text::<_Key, _Map, _Language>);

		app
	}

	#[test]
	fn call_language_server_with_for_correct_key() {
		let mut app = setup(Setup {
			map: _Map::new_mock(|mock| {
				mock.expect_map_forward().return_const(KeyCode::KeyB);
			}),
			language: _Language::new_mock(|mock| {
				mock.expect_ui_text_for()
					.times(1)
					.with(eq(KeyCode::KeyB))
					.return_const(UIText::Unmapped);
			}),
		});
		app.world_mut()
			.spawn(KeyCodeTextInsertCommand::new(_Key, TextStyle::default()));

		app.update();
	}

	#[test]
	fn spawn_text_bundle() {
		let mut app = setup(Setup {
			map: _Map::new_mock(|mock| {
				mock.expect_map_forward().return_const(KeyCode::KeyB);
			}),
			language: _Language::new_mock(|mock| {
				mock.expect_ui_text_for()
					.return_const(UIText::String("my text".to_owned()));
			}),
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand::new(_Key, TextStyle::default()))
			.id();

		app.update();

		let text = app.world().entity(text);

		assert_bundle!(TextBundle, &app, text);
	}

	#[test]
	fn spawn_text_bundle_text() {
		let mut app = setup(Setup {
			language: _Language::new_mock(|mock| {
				mock.expect_ui_text_for()
					.return_const(UIText::String("my text".to_owned()));
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand::new(_Key, TextStyle::default()))
			.id();

		app.update();

		let text = app.world().entity(text);
		let default_style = TextStyle::default();

		assert_bundle!(
			TextBundle,
			&app,
			text,
			With::assert(|text: &Text| {
				assert_eq!(
					text.sections
						.iter()
						.map(|t| (
							t.value.clone(),
							t.style.color,
							t.style.font.clone(),
							t.style.font_size
						))
						.collect::<Vec<_>>(),
					vec![(
						"my text".to_owned(),
						default_style.color,
						default_style.font.clone(),
						default_style.font_size,
					)]
				);
			})
		);
	}

	#[test]
	fn spawn_text_bundle_style() {
		let text_style = TextStyle {
			font: Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
			font_size: 42.,
			color: Color::BLACK,
		};
		let mut app = setup(Setup {
			language: _Language::new_mock(|mock| {
				mock.expect_ui_text_for()
					.return_const(UIText::String("my text".to_owned()));
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand::new(_Key, text_style.clone()))
			.id();

		app.update();

		let text = app.world().entity(text);

		assert_bundle!(
			TextBundle,
			&app,
			text,
			With::assert(|text: &Text| {
				assert_eq!(
					text.sections
						.iter()
						.map(|t| (
							t.value.clone(),
							t.style.color,
							t.style.font.clone(),
							t.style.font_size
						))
						.collect::<Vec<_>>(),
					vec![(
						"my text".to_owned(),
						text_style.color,
						text_style.font.clone(),
						text_style.font_size,
					)]
				);
			})
		);
	}

	#[test]
	fn remove_insert_command() {
		let mut app = setup(Setup {
			language: _Language::new_mock(|mock| {
				mock.expect_ui_text_for()
					.return_const(UIText::String("my text".to_owned()));
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand::new(_Key, TextStyle::default()))
			.id();

		app.update();

		let text = app.world().entity(text);

		assert_eq!(None, text.get::<KeyCodeTextInsertCommand<_Key>>());
	}

	#[test]
	fn remove_insert_command_even_when_key_cannot_be_mapped() {
		let mut app = setup(Setup {
			language: _Language::new_mock(|mock| {
				mock.expect_ui_text_for().return_const(UIText::Unmapped);
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand::new(_Key, TextStyle::default()))
			.id();

		app.update();

		let text = app.world().entity(text);

		assert_eq!(None, text.get::<KeyCodeTextInsertCommand<_Key>>());
	}
}
