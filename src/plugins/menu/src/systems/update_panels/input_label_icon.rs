use crate::components::{
	icon::{Icon, IconImage},
	input_label::InputLabel,
};
use bevy::prelude::*;
use common::{
	tools::action_key::user_input::UserInput,
	traits::{
		handles_localization::{LocalizeToken, Token, localized::Localized},
		key_mappings::GetInput,
		thread_safe::ThreadSafe,
	},
};
use std::path::{Path, PathBuf};

impl<TKey> InputLabel<TKey>
where
	TKey: Copy + ThreadSafe,
{
	#[allow(clippy::type_complexity)]
	pub fn icon<TMap, TLanguageServer>(
		icon_root_path: impl Into<PathBuf>,
	) -> impl Fn(
		Commands,
		Res<TMap>,
		ResMut<TLanguageServer>,
		Query<(Entity, &InputLabel<TKey>), Added<InputLabel<TKey>>>,
	)
	where
		TMap: Resource + GetInput<TKey, UserInput>,
		TLanguageServer: Resource + LocalizeToken,
	{
		let root = icon_root_path.into();

		move |mut commands, key_map, mut language_server, mut labels| {
			let key_map = key_map.as_ref();

			for (entity, label) in &mut labels {
				let Some(entity) = commands.get_entity(entity) else {
					continue;
				};
				insert_icon(&root, entity, key_map, language_server.as_mut(), label);
			}
		}
	}
}

fn insert_icon<TMap, TLanguageServer, TKey>(
	root: &Path,
	mut entity: EntityCommands,
	key_map: &TMap,
	language_server: &mut TLanguageServer,
	label: &InputLabel<TKey>,
) where
	TMap: GetInput<TKey, UserInput>,
	TLanguageServer: LocalizeToken,
	TKey: Copy,
{
	let key = key_map.get_input(label.key);
	let Localized(description) = language_server.localize_token(key).or_token();
	let Token(token) = Token::from(key);
	let path = root.join(format!("{token}.png"));

	entity.insert(Icon {
		description,
		image: IconImage::Path(path),
	});
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::icon::{Icon, IconImage};
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
	use std::path::PathBuf;

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

		app.add_systems(
			Update,
			InputLabel::<_Key>::icon::<_Map, _LanguageServer>("icon/root/path"),
		);
		app.insert_resource(map);
		app.insert_resource(language_server);

		app
	}

	#[test]
	fn add_icon() {
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
		let id = app.world_mut().spawn(InputLabel { key: _Key }).id();

		app.update();

		let Token(token) = Token::from(UserInput::from(KeyCode::ArrowUp));
		assert_eq!(
			Some(&Icon {
				description: String::from("IIIIII"),
				image: IconImage::Path(
					PathBuf::from("icon/root/path").join(format!("{token}.png"))
				)
			}),
			app.world().entity(id).get::<Icon>()
		)
	}

	#[test]
	fn do_not_add_icon_if_not_added() {
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
		let id = app.world_mut().spawn(InputLabel { key: _Key }).id();

		app.update();
		app.world_mut().entity_mut(id).remove::<Icon>();
		app.update();

		assert_eq!(None, app.world().entity(id).get::<Icon>())
	}
}
