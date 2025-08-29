use crate::components::{
	icon::{Icon, IconImage},
	input_label::InputLabel,
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::GetMut,
		handles_localization::{LocalizeToken, Token},
		key_mappings::GetInput,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
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
		ZyheedaCommands,
		Res<TMap>,
		Res<TLanguageServer>,
		Query<(Entity, &InputLabel<TKey>), Added<InputLabel<TKey>>>,
	)
	where
		TMap: Resource + GetInput<TKey>,
		TMap::TInput: Into<Token>,
		TLanguageServer: Resource + LocalizeToken,
	{
		let root = icon_root_path.into();

		move |mut commands, key_map, language_server, mut labels| {
			let key_map = key_map.as_ref();

			for (entity, label) in &mut labels {
				let Some(entity) = commands.get_mut(&entity) else {
					continue;
				};
				insert_icon(&root, entity, key_map, language_server.as_ref(), label);
			}
		}
	}
}

fn insert_icon<TMap, TLanguageServer, TKey>(
	root: &Path,
	mut entity: ZyheedaEntityCommands,
	key_map: &TMap,
	language_server: &TLanguageServer,
	label: &InputLabel<TKey>,
) where
	TMap: GetInput<TKey>,
	TMap::TInput: Into<Token>,
	TLanguageServer: LocalizeToken,
	TKey: Copy,
{
	let key = key_map.get_input(label.key);
	let token = key.into();
	let localized = language_server.localize_token(token.clone()).or_token();
	let Token(token) = token;
	let path = root.join(format!("{token}.png"));

	entity.try_insert(Icon {
		localized,
		image: IconImage::Path(path),
	});
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::icon::{Icon, IconImage};
	use bevy::app::{App, Update};
	use common::{
		tools::action_key::user_input::UserInput,
		traits::handles_localization::{LocalizationResult, Token, localized::Localized},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Clone, Copy)]
	struct _Key;

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl GetInput<_Key> for _Map {
		type TInput = UserInput;

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
		fn localize_token<TToken>(&self, token: TToken) -> LocalizationResult
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
					.with(eq(Token::from(UserInput::from(KeyCode::ArrowUp))))
					.return_const(LocalizationResult::Ok(Localized::from("IIIIII")));
			}),
		);
		let id = app.world_mut().spawn(InputLabel { key: _Key }).id();

		app.update();

		let Token(token) = Token::from(UserInput::from(KeyCode::ArrowUp));
		assert_eq!(
			Some(&Icon {
				localized: Localized::from("IIIIII"),
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
					.with(eq(Token::from(UserInput::from(KeyCode::ArrowUp))))
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
