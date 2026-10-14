use crate::components::{
	icon::{Icon, IconStackItem},
	input_label::InputLabel,
	label::UILabel,
};
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

				for (index, key) in keys.enumerate() {
					commands.spawn((
						IconStackItem::of(entity).with_index(index),
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
	use bevy::app::{App, Update};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
	use testing::{NestedMocks, SingleThreadedApp, assert_related_count};

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
		let [child] = assert_related_count!(IconStackItem, 1, app, id);
		assert_eq!(
			Some(&Icon::ImagePath(
				PathBuf::from("icon/root/path").join(format!("{item}.png"))
			)),
			child.get::<Icon>(),
		);
	}

	#[test]
	fn add_combination_icons() -> Result<(), InputRepeatedError> {
		let input = UserInput::Combined(UserInputCombination::try_from([
			RawInput::KeyCode(KeyCode::KeyA),
			RawInput::KeyCode(KeyCode::KeyB),
		])?);
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input()
				.times(1)
				.with(eq(HandSlot::Left))
				.return_const(input);
		}));
		let id = app
			.world_mut()
			.spawn(InputLabel {
				key: HandSlot::Left,
			})
			.id();

		app.update();

		let token = Token::from(input);
		let token_items = token.iter().collect::<Vec<_>>();
		let items = match token_items[..] {
			[TokenItem::Key(a), TokenItem::Raw(..), TokenItem::Key(b)] => [a, b],
			_ => panic!("FAULTY ASSUMPTION"),
		};
		let children = assert_related_count!(IconStackItem, 2, app, id);
		assert_eq!(
			items.map(|item| Some(Icon::ImagePath(
				PathBuf::from("icon/root/path").join(format!("{item}.png"))
			))),
			children.map(|c| c.get::<Icon>().cloned())
		);
		Ok(())
	}

	#[test]
	fn add_combination_icons_index() -> Result<(), InputRepeatedError> {
		let input = UserInput::Combined(UserInputCombination::try_from([
			RawInput::KeyCode(KeyCode::KeyA),
			RawInput::KeyCode(KeyCode::KeyB),
		])?);
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input()
				.times(1)
				.with(eq(HandSlot::Left))
				.return_const(input);
		}));
		let id = app
			.world_mut()
			.spawn(InputLabel {
				key: HandSlot::Left,
			})
			.id();

		app.update();

		let children = assert_related_count!(IconStackItem, 2, app, id);
		assert_eq!(
			[
				Some(&IconStackItem { of: id, index: 0 }),
				Some(&IconStackItem { of: id, index: 1 })
			],
			children.map(|c| c.get::<IconStackItem>())
		);
		Ok(())
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

		let [child] = assert_related_count!(IconStackItem, 1, app, id);
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
		let [child] = assert_related_count!(IconStackItem, 1, app, id);
		let child = child.id();
		app.world_mut().entity_mut(child).remove::<Icon>();
		app.update();

		assert_eq!(None, app.world().entity(child).get::<Icon>())
	}
}
