use crate::{components::input_label::InputLabel, traits::colors::HasPanelColors};
use bevy::prelude::*;
use common::{
	tools::action_key::user_input::UserInput,
	traits::{
		handles_localization::LocalizeToken,
		key_mappings::GetInput,
		thread_safe::ThreadSafe,
	},
};

type InputLabels<'a, T, TKey> = (&'a InputLabel<T, TKey>, &'a mut Text);

impl<T, TKey> InputLabel<T, TKey>
where
	T: HasPanelColors + ThreadSafe,
	TKey: Copy + ThreadSafe,
{
	pub fn ui<TMap, TLanguageServer>(
		key_map: Res<TMap>,
		mut language_server: ResMut<TLanguageServer>,
		mut labels: Query<InputLabels<T, TKey>, Added<InputLabel<T, TKey>>>,
	) where
		TMap: Resource + GetInput<TKey, UserInput>,
		TLanguageServer: Resource + LocalizeToken,
	{
		let key_map = key_map.as_ref();

		for (label, text) in &mut labels {
			update_text(key_map, language_server.as_mut(), label, text);
		}
	}
}

fn update_text<TMap, TLanguageServer, T, TKey>(
	key_map: &TMap,
	language_server: &mut TLanguageServer,
	label: &InputLabel<T, TKey>,
	mut text: Mut<Text>,
) where
	TMap: GetInput<TKey, UserInput>,
	TLanguageServer: LocalizeToken,
	T: HasPanelColors,
	TKey: Copy,
{
	let key = key_map.get_input(label.key);
	let localized = language_server.localize_token(key).or_token();
	*text = Text::from(localized);
}

#[cfg(test)]
mod tests {
	use crate::traits::colors::PanelColors;

	use super::*;
	use bevy::app::{App, Update};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{
			handles_localization::{LocalizationResult, Token, localized::Localized},
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	struct _T;

	impl HasPanelColors for _T {
		const PANEL_COLORS: PanelColors = PanelColors::DEFAULT;
	}

	#[derive(Clone, Copy)]
	struct _Key;

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl GetInput<_Key, UserInput> for _Map {
		fn get_input(&self, value: _Key) -> UserInput {
			self.mock.get_input(value)
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

		app.add_systems(Update, InputLabel::<_T, _Key>::ui::<_Map, _LanguageServer>);
		app.insert_resource(map);
		app.insert_resource(language_server);

		app
	}

	#[test]
	fn add_section_to_text() {
		let mut app = setup(
			_Map::new().with_mock(|mock| {
				mock.expect_get_input()
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
			.spawn((InputLabel::<_T, _Key>::new(_Key), Text::default()))
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
				mock.expect_get_input()
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
				InputLabel::<_T, _Key>::new(_Key),
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
