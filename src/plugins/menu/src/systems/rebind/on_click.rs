use crate::{Input, KeyBind, Rebinding};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};

impl<TAction> KeyBind<Input<TAction>>
where
	TAction: Copy + ThreadSafe,
{
	pub(crate) fn rebind_on_click(
		mut commands: ZyheedaCommands,
		key_binds: Query<(Entity, &Self, &Interaction, Option<&Children>)>,
	) {
		for (entity, KeyBind(input), interaction, children) in &key_binds {
			if interaction != &Interaction::Pressed {
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(KeyBind(Rebinding(*input)));
				e.try_remove::<Self>();
			});

			let Some(children) = children else {
				continue;
			};

			for child in children {
				commands.try_apply_on(child, |e| e.try_despawn());
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Rebinding;
	use common::tools::action_key::user_input::UserInput;
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Action;

	#[derive(Component, Debug, PartialEq)]
	struct _Child;

	impl _Child {
		fn spawn_nested_once(key_bind: &mut ChildSpawner) {
			key_bind.spawn(_Child).with_children(|child| {
				child.spawn(_Child);
			});
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, KeyBind::<Input<_Action>>::rebind_on_click);

		app
	}

	#[test]
	fn insert_rebind() {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		let entity = app
			.world_mut()
			.spawn((
				KeyBind(Input {
					action: _Action,
					input: UserInput::MouseButton(MouseButton::Left),
				}),
				Interaction::Pressed,
			))
			.insert(ChildOf(parent))
			.id();

		app.update();

		assert_eq!(
			Some(&KeyBind(Rebinding(Input {
				action: _Action,
				input: UserInput::MouseButton(MouseButton::Left)
			}))),
			app.world()
				.entity(entity)
				.get::<KeyBind<Rebinding<_Action>>>(),
		);
	}

	#[test]
	fn remove_input() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				KeyBind(Input {
					action: _Action,
					input: UserInput::MouseButton(MouseButton::Left),
				}),
				Interaction::Pressed,
			))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<KeyBind<Input<_Action>>>(),
		);
	}

	#[test]
	fn remove_all_input_children() {
		let mut app = setup();
		app.world_mut()
			.spawn((
				KeyBind(Input {
					action: _Action,
					input: UserInput::MouseButton(MouseButton::Left),
				}),
				Interaction::Pressed,
			))
			.with_children(_Child::spawn_nested_once);

		app.update();

		let mut children = app.world_mut().query_filtered::<(), With<_Child>>();
		assert_count!(0, children.iter(app.world()));
	}

	#[test]
	fn do_nothing_if_not_pressed() {
		let mut app = setup();
		let entities = [
			app.world_mut()
				.spawn((
					KeyBind(Input {
						action: _Action,
						input: UserInput::MouseButton(MouseButton::Left),
					}),
					Interaction::None,
				))
				.with_children(_Child::spawn_nested_once)
				.id(),
			app.world_mut()
				.spawn((
					KeyBind(Input {
						action: _Action,
						input: UserInput::MouseButton(MouseButton::Left),
					}),
					Interaction::Hovered,
				))
				.with_children(_Child::spawn_nested_once)
				.id(),
		];

		app.update();

		let mut children = app.world_mut().query_filtered::<(), With<_Child>>();
		assert_eq!(
			([true, true], [false, false], 4),
			(
				app.world()
					.entity(entities)
					.map(|e| e.contains::<KeyBind<Input<_Action>>>()),
				app.world()
					.entity(entities)
					.map(|e| e.contains::<KeyBind<Rebinding<_Action>>>()),
				children.iter(app.world()).count()
			)
		);
	}
}
