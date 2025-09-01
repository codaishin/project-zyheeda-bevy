use crate::components::{icon::Icon, input_label::InputLabel, label::UILabel};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		handles_localization::Token,
		key_mappings::GetInput,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::path::PathBuf;

impl<TKey> InputLabel<TKey>
where
	TKey: Copy + ThreadSafe,
{
	pub fn icon<TMap>(
		icon_root_path: impl Into<PathBuf>,
	) -> impl Fn(ZyheedaCommands, Res<TMap>, Labels<TKey>)
	where
		TMap: Resource + GetInput<TKey>,
		TMap::TInput: Into<Token>,
	{
		let root = icon_root_path.into();

		move |mut commands, key_map, labels| {
			let key_map = key_map.as_ref();

			for (entity, label) in &labels {
				commands.try_apply_on(&entity, |mut e| {
					let key = key_map.get_input(label.key);
					let token = key.into();
					let image_name = &*token;
					let path = root.join(format!("{image_name}.png"));

					e.try_insert((UILabel(token), Icon::ImagePath(path)));
				});
			}
		}
	}
}

type Labels<'w, 's, 'a, TKey> =
	Query<'w, 's, (Entity, &'a InputLabel<TKey>), Added<InputLabel<TKey>>>;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::icon::{Icon, IconFallbackLabel};
	use bevy::app::{App, Update};
	use common::{tools::action_key::user_input::UserInput, traits::handles_localization::Token};
	use macros::NestedMocks;
	use mockall::automock;
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

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, InputLabel::<_Key>::icon::<_Map>("icon/root/path"));
		app.insert_resource(map);

		app
	}

	#[test]
	fn add_icon() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input()
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app.world_mut().spawn(InputLabel { key: _Key }).id();

		app.update();

		let token = &*Token::from(UserInput::from(KeyCode::ArrowUp));
		assert_eq!(
			Some(&Icon::ImagePath(
				PathBuf::from("icon/root/path").join(format!("{token}.png"))
			)),
			app.world().entity(id).get::<Icon>(),
		);
	}

	#[test]
	fn add_icon_fallback_label() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input()
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app.world_mut().spawn(InputLabel { key: _Key }).id();

		app.update();

		assert_eq!(
			Some(&UILabel(Token::from(UserInput::from(KeyCode::ArrowUp)))),
			app.world().entity(id).get::<IconFallbackLabel>(),
		);
	}

	#[test]
	fn do_not_add_icon_if_not_added() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input()
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app.world_mut().spawn(InputLabel { key: _Key }).id();

		app.update();
		app.world_mut().entity_mut(id).remove::<Icon>();
		app.update();

		assert_eq!(None, app.world().entity(id).get::<Icon>())
	}
}
