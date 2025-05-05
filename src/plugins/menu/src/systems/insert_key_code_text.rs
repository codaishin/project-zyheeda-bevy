use crate::components::key_code_text_insert_command::UserInputTextInsertCommand;
use bevy::prelude::*;
use common::{
	tools::keys::user_input::UserInput,
	traits::{
		handles_localization::LocalizeToken,
		key_mappings::GetUserInput,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

pub(crate) fn insert_user_input_text<TKey, TKeyMap, TLanguageServer>(
	mut commands: Commands,
	insert_commands: Query<(Entity, &UserInputTextInsertCommand<TKey>)>,
	key_map: Res<TKeyMap>,
	mut language_server: ResMut<TLanguageServer>,
) where
	TKey: Copy + Sync + Send + 'static,
	TKeyMap: Resource + GetUserInput<TKey, UserInput>,
	TLanguageServer: Resource + LocalizeToken,
{
	for (entity, insert_command) in &insert_commands {
		let key = key_map.get_key_code(insert_command.key);
		let localized = language_server.localize_token(key).or_token();
		commands.try_insert_on(
			entity,
			(
				Text::new(localized),
				insert_command.font.clone(),
				insert_command.color,
				insert_command.layout,
			),
		);
		commands.try_remove_from::<UserInputTextInsertCommand<TKey>>(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{
			handles_localization::{LocalizationResult, Token, localized::Localized},
			nested_mock::NestedMocks,
		},
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
	impl GetUserInput<_Key, UserInput> for _Map {
		fn get_key_code(&self, value: _Key) -> UserInput {
			self.mock.get_key_code(value)
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _LanguageServer {
		mock: Mock_LanguageServer,
	}

	#[automock]
	impl LocalizeToken for _LanguageServer {
		fn localize_token<TToken>(&mut self, token: TToken) -> LocalizationResult
		where
			TToken: Into<Token> + 'static,
		{
			self.mock.localize_token(token)
		}
	}

	struct Setup {
		map: _Map,
		language: _LanguageServer,
	}

	impl Default for Setup {
		fn default() -> Self {
			Self {
				map: _Map::new().with_mock(|mock| {
					mock.expect_get_key_code()
						.return_const(UserInput::from(KeyCode::KeyA));
				}),
				language: _LanguageServer::new().with_mock(|mock| {
					mock.expect_localize_token::<KeyCode>()
						.returning(|token| LocalizationResult::Error(Token::from(token).failed()));
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
			insert_user_input_text::<_Key, _Map, _LanguageServer>,
		);

		app
	}

	#[test]
	fn call_language_server_for_correct_key() {
		let mut app = setup(Setup {
			map: _Map::new().with_mock(|mock| {
				mock.expect_get_key_code()
					.return_const(UserInput::from(KeyCode::KeyB));
			}),
			language: _LanguageServer::new().with_mock(|mock| {
				mock.expect_localize_token()
					.times(1)
					.with(eq(UserInput::from(KeyCode::KeyB)))
					.returning(|token| LocalizationResult::Error(Token::from(token).failed()));
			}),
		});
		app.world_mut().spawn(UserInputTextInsertCommand {
			key: _Key,
			..default()
		});

		app.update();
	}

	#[test]
	fn spawn_text_bundle() {
		let mut app = setup(Setup {
			map: _Map::new().with_mock(|mock| {
				mock.expect_get_key_code()
					.return_const(UserInput::from(KeyCode::KeyB));
			}),
			language: _LanguageServer::new().with_mock(|mock| {
				mock.expect_localize_token::<UserInput>()
					.return_const(LocalizationResult::Ok(Localized::from("my text")));
			}),
		});
		let text = app
			.world_mut()
			.spawn(UserInputTextInsertCommand::<_Key>::default())
			.id();

		app.update();

		assert!(app.world().entity(text).get::<Text>().is_some());
	}

	#[test]
	fn spawn_text() {
		let mut app = setup(Setup {
			language: _LanguageServer::new().with_mock(|mock| {
				mock.expect_localize_token::<UserInput>()
					.return_const(LocalizationResult::Ok(Localized::from("my text")));
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(UserInputTextInsertCommand {
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
			language: _LanguageServer::new().with_mock(|mock| {
				mock.expect_localize_token::<UserInput>()
					.return_const(LocalizationResult::Ok(Localized::from("my text")));
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(UserInputTextInsertCommand::<_Key>::default())
			.id();

		app.update();

		let text = app.world().entity(text);

		assert_eq!(None, text.get::<UserInputTextInsertCommand<_Key>>());
	}

	#[test]
	fn remove_insert_command_even_when_key_cannot_be_mapped() {
		let mut app = setup(Setup {
			language: _LanguageServer::new().with_mock(|mock| {
				mock.expect_localize_token::<UserInput>()
					.returning(|key| LocalizationResult::Error(Token::from(key).failed()));
			}),
			..default()
		});
		let text = app
			.world_mut()
			.spawn(UserInputTextInsertCommand {
				key: _Key,
				..default()
			})
			.id();

		app.update();

		let text = app.world().entity(text);

		assert_eq!(None, text.get::<UserInputTextInsertCommand<_Key>>());
	}
}
