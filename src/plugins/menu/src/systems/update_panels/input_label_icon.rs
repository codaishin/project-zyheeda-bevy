use crate::components::{icon::Icon, input_label::InputLabel, label::UILabel};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	traits::{accessors::get::TryApplyOn, handles_input::GetInput, handles_localization::Token},
	zyheeda_commands::ZyheedaCommands,
};
use std::{ops::Deref, path::PathBuf};

impl InputLabel {
	pub fn icon<TInput>(
		icon_root_path: impl Into<PathBuf>,
	) -> impl Fn(ZyheedaCommands, StaticSystemParam<TInput>, Labels)
	where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetInput>,
	{
		let root = icon_root_path.into();

		move |mut commands, key_map, labels| {
			let key_map = key_map.deref();

			for (entity, label) in &labels {
				commands.try_apply_on(&entity, |mut e| {
					let key = key_map.get_input(label.key);
					let token = Token::from(key);
					let image_name = &*token;
					let path = root.join(format!("{image_name}.png"));

					e.try_insert((UILabel(token), Icon::ImagePath(path)));
				});
			}
		}
	}
}

type Labels<'w, 's, 'a> = Query<'w, 's, (Entity, &'a InputLabel), Added<InputLabel>>;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::icon::Icon;
	use bevy::app::{App, Update};
	use common::{
		tools::action_key::{ActionKey, slot::PlayerSlot, user_input::UserInput},
		traits::handles_localization::Token,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetInput for _Input {
		fn get_input<TAction>(&self, value: TAction) -> UserInput
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.mock.get_input(value)
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, InputLabel::icon::<Res<_Input>>("icon/root/path"));
		app.insert_resource(input);

		app
	}

	#[test]
	fn add_icon() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input()
				.times(1)
				.with(eq(PlayerSlot::UPPER_L))
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app
			.world_mut()
			.spawn(InputLabel {
				key: PlayerSlot::UPPER_L,
			})
			.id();

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
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input::<PlayerSlot>()
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app
			.world_mut()
			.spawn(InputLabel {
				key: PlayerSlot::UPPER_L,
			})
			.id();

		app.update();

		assert_eq!(
			Some(&UILabel(Token::from(UserInput::from(KeyCode::ArrowUp)))),
			app.world().entity(id).get::<UILabel<Token>>(),
		);
	}

	#[test]
	fn do_not_add_icon_if_not_added() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input::<PlayerSlot>()
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app
			.world_mut()
			.spawn(InputLabel {
				key: PlayerSlot::UPPER_L,
			})
			.id();

		app.update();
		app.world_mut().entity_mut(id).remove::<Icon>();
		app.update();

		assert_eq!(None, app.world().entity(id).get::<Icon>())
	}
}
