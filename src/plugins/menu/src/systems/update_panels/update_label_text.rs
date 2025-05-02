use crate::components::Label;
use bevy::prelude::*;
use common::{
	tools::keys::{slot::SlotKey, user_input::UserInput},
	traits::{handles_localization::LocalizeToken, key_mappings::GetUserInput},
};

type Labels<'a, T> = (&'a Label<T, SlotKey>, &'a mut Text);

pub fn update_label_text<
	TMap: Resource + GetUserInput<SlotKey, UserInput>,
	TLanguageServer: Resource + LocalizeToken,
	TComponent: Sync + Send + 'static,
>(
	key_map: Res<TMap>,
	mut language_server: ResMut<TLanguageServer>,
	mut labels: Query<Labels<TComponent>, Added<Label<TComponent, SlotKey>>>,
) {
	let key_map = key_map.as_ref();

	for (label, text) in &mut labels {
		update_text(key_map, language_server.as_mut(), label, text);
	}
}

fn update_text<TMap, TLanguageServer, TComponent>(
	key_map: &TMap,
	language_server: &mut TLanguageServer,
	label: &Label<TComponent, SlotKey>,
	mut text: Mut<Text>,
) where
	TMap: GetUserInput<SlotKey, UserInput>,
	TLanguageServer: LocalizeToken,
{
	let key = key_map.get_key_code(label.key);
	let localized = language_server.localize_token(key).or_token();
	*text = Text::from(localized);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::keys::slot::Side,
		traits::{
			handles_localization::{LocalizationResult, Token, localized::Localized},
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	struct _T;

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl GetUserInput<SlotKey, UserInput> for _Map {
		fn get_key_code(&self, value: SlotKey) -> UserInput {
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

	fn setup(map: _Map, language_server: _LanguageServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, update_label_text::<_Map, _LanguageServer, _T>);
		app.insert_resource(map);
		app.insert_resource(language_server);

		app
	}

	#[test]
	fn add_section_to_text() {
		let mut app = setup(
			_Map::new().with_mock(|mock| {
				mock.expect_get_key_code()
					.return_const(UserInput::from(KeyCode::ArrowUp));
			}),
			_LanguageServer::new().with_mock(|mock| {
				mock.expect_localize_token()
					.with(eq(UserInput::from(KeyCode::ArrowUp)))
					.return_const(LocalizationResult::Ok(Localized::from("IIIIII")));
			}),
		);
		let id = app
			.world_mut()
			.spawn((
				Label::<_T, SlotKey>::new(SlotKey::BottomHand(Side::Right)),
				Text::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some("IIIIII"),
			app.world()
				.entity(id)
				.get::<Text>()
				.map(|Text(text)| text.as_str())
		)
	}

	#[test]
	fn override_original() {
		let mut app = setup(
			_Map::new().with_mock(|mock| {
				mock.expect_get_key_code()
					.return_const(UserInput::from(KeyCode::ArrowUp));
			}),
			_LanguageServer::new().with_mock(|mock| {
				mock.expect_localize_token()
					.with(eq(UserInput::from(KeyCode::ArrowUp)))
					.return_const(LocalizationResult::Ok(Localized::from("IIIIII")));
			}),
		);
		let id = app
			.world_mut()
			.spawn((
				Label::<_T, SlotKey>::new(SlotKey::BottomHand(Side::Right)),
				Text::new("OVERRIDE THIS"),
			))
			.id();

		app.update();

		assert_eq!(
			Some("IIIIII"),
			app.world()
				.entity(id)
				.get::<Text>()
				.map(|Text(text)| text.as_str())
		)
	}
}
