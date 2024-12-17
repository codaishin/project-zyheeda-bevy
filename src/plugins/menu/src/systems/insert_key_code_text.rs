use crate::components::key_code_text_insert_command::KeyCodeTextInsertCommand;
use bevy::prelude::*;
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
		let key = key_map.map_forward(insert_command.key);
		if let UIText::String(text) = language_server.ui_text_for(&key) {
			commands.try_insert_on(
				entity,
				(
					Text::new(text),
					insert_command.font.clone(),
					insert_command.color,
					insert_command.layout,
				),
			);
		};
		commands.try_remove_from::<KeyCodeTextInsertCommand<TKey>>(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{get_ui_text::UIText, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Clone, Copy, Debug, PartialEq, Default)]
	struct _Key;

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl MapForward<_Key, KeyCode> for _Map {
		fn map_forward(&self, value: _Key) -> KeyCode {
			self.mock.map_forward(value)
		}
	}

	#[derive(Resource, NestedMocks)]
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
				map: _Map::new().with_mock(|mock| {
					mock.expect_map_forward().return_const(KeyCode::KeyA);
				}),
				language: _Language::new().with_mock(|mock| {
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
	fn call_language_server_for_correct_key() {
		let mut app = setup(Setup {
			map: _Map::new().with_mock(|mock| {
				mock.expect_map_forward().return_const(KeyCode::KeyB);
			}),
			language: _Language::new().with_mock(|mock| {
				mock.expect_ui_text_for()
					.times(1)
					.with(eq(KeyCode::KeyB))
					.return_const(UIText::Unmapped);
			}),
		});
		app.world_mut().spawn(KeyCodeTextInsertCommand {
			key: _Key,
			..default()
		});

		app.update();
	}

	#[test]
	fn spawn_text_bundle() {
		let mut app = setup(Setup {
			map: _Map::new().with_mock(|mock| {
				mock.expect_map_forward().return_const(KeyCode::KeyB);
			}),
			language: _Language::new().with_mock(|mock| {
				mock.expect_ui_text_for()
					.return_const(UIText::String("my text".to_owned()));
			}),
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand::<_Key>::default())
			.id();

		app.update();

		assert!(app.world().entity(text).get::<Text>().is_some());
	}

	#[test]
	fn spawn_text() {
		let mut app = setup(Setup {
			language: _Language::new().with_mock(|mock| {
				mock.expect_ui_text_for()
					.return_const(UIText::String("my text".to_owned()));
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand {
				key: _Key,
				font: TextFont {
					font_size: 11.,
					..default()
				},
				color: TextColor(Color::linear_rgb(0.5, 0.1, 0.2)),
				layout: TextLayout {
					justify: JustifyText::Justified,
					..default()
				},
			})
			.id();

		app.update();

		let text = app.world().entity(text);
		assert_eq!(
			(
				Some("my text"),
				Some(&11.),
				Some(&Color::linear_rgb(0.5, 0.1, 0.2)),
				Some(&JustifyText::Justified)
			),
			(
				text.get::<Text>().map(|Text(text)| text.as_str()),
				text.get::<TextFont>()
					.map(|TextFont { font_size, .. }| font_size),
				text.get::<TextColor>().map(|TextColor(color)| color),
				text.get::<TextLayout>()
					.map(|TextLayout { justify, .. }| justify)
			)
		);
	}

	#[test]
	fn remove_insert_command() {
		let mut app = setup(Setup {
			language: _Language::new().with_mock(|mock| {
				mock.expect_ui_text_for()
					.return_const(UIText::String("my text".to_owned()));
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand::<_Key>::default())
			.id();

		app.update();

		let text = app.world().entity(text);

		assert_eq!(None, text.get::<KeyCodeTextInsertCommand<_Key>>());
	}

	#[test]
	fn remove_insert_command_even_when_key_cannot_be_mapped() {
		let mut app = setup(Setup {
			language: _Language::new().with_mock(|mock| {
				mock.expect_ui_text_for().return_const(UIText::Unmapped);
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(KeyCodeTextInsertCommand {
				key: _Key,
				..default()
			})
			.id();

		app.update();

		let text = app.world().entity(text);

		assert_eq!(None, text.get::<KeyCodeTextInsertCommand<_Key>>());
	}
}
