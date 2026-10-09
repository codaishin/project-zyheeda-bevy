use crate::components::{icon::Icon, input_label::InputLabel, label::UILabel};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::prelude::*;
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
				let key = key_map.get_input(label.key);
				let token = Token::from(key);
				let keys = token.items().filter_map(|item| match item {
					TokenItem::Key(key) => Some(key),
					TokenItem::Raw(..) => None,
				});

				for key in keys {
					commands.spawn((
						ChildOf(entity),
						UILabel(Token::from(key)),
						Icon::ImagePath(root.join(format!("{key}.png"))),
					));
				}
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
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
	use testing::{NestedMocks, SingleThreadedApp, assert_children_count};

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
				.with(eq(HandSlot::Left))
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app
			.world_mut()
			.spawn(InputLabel {
				key: HandSlot::Left,
			})
			.id();

		app.update();

		let token = Token::from(UserInput::from(KeyCode::ArrowUp));
		let Some(TokenItem::Key(item)) = token.items().next() else {
			panic!("FAULTY ASSUMPTION")
		};
		let [child] = assert_children_count!(1, app, id);
		assert_eq!(
			Some(&Icon::ImagePath(
				PathBuf::from("icon/root/path").join(format!("{item}.png"))
			)),
			child.get::<Icon>(),
		);
	}

	#[test]
	fn add_icon_fallback_label() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input::<HandSlot>()
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app
			.world_mut()
			.spawn(InputLabel {
				key: HandSlot::Left,
			})
			.id();

		app.update();

		let [child] = assert_children_count!(1, app, id);
		assert_eq!(
			Some(&UILabel(Token::from(UserInput::from(KeyCode::ArrowUp)))),
			child.get::<UILabel<Token>>(),
		);
	}

	#[test]
	fn do_not_add_icon_if_not_added() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input::<HandSlot>()
				.return_const(UserInput::from(KeyCode::ArrowUp));
		}));
		let id = app
			.world_mut()
			.spawn(InputLabel {
				key: HandSlot::Left,
			})
			.id();

		app.update();
		let [child] = assert_children_count!(1, app, id);
		let child = child.id();
		app.world_mut().entity_mut(child).remove::<Icon>();
		app.update();

		assert_eq!(None, app.world().entity(child).get::<Icon>())
	}
}
