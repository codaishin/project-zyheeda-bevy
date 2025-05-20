use crate::{Input, KeyBind, Rebinding};
use bevy::prelude::*;
use common::traits::{thread_safe::ThreadSafe, try_despawn::TryDespawn};

impl<TAction, TInput> KeyBind<Input<TAction, TInput>>
where
	TAction: Copy + ThreadSafe,
	TInput: Copy + ThreadSafe,
{
	pub(crate) fn rebind_on_click(
		mut commands: Commands,
		key_binds: Query<(Entity, &Self, &Interaction, Option<&Children>)>,
	) {
		for (entity, KeyBind(input), interaction, children) in &key_binds {
			if interaction != &Interaction::Pressed {
				continue;
			}

			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.try_insert(KeyBind(Rebinding(*input)));
			entity.remove::<Self>();

			let Some(children) = children else {
				continue;
			};

			for child in children {
				commands.try_despawn(*child);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Rebinding;
	use common::{assert_count, test_tools::utils::SingleThreadedApp};

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Action;

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Input;

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

		app.add_systems(Update, KeyBind::<Input<_Action, _Input>>::rebind_on_click);

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
					input: _Input,
				}),
				Interaction::Pressed,
			))
			.insert(ChildOf(parent))
			.id();

		app.update();

		assert_eq!(
			Some(&KeyBind(Rebinding(Input {
				action: _Action,
				input: _Input
			}))),
			app.world()
				.entity(entity)
				.get::<KeyBind<Rebinding<_Action, _Input>>>(),
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
					input: _Input,
				}),
				Interaction::Pressed,
			))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<KeyBind<Input<_Action, _Input>>>(),
		);
	}

	#[test]
	fn remove_all_input_children() {
		let mut app = setup();
		app.world_mut()
			.spawn((
				KeyBind(Input {
					action: _Action,
					input: _Input,
				}),
				Interaction::Pressed,
			))
			.with_children(_Child::spawn_nested_once);

		app.update();

		assert_count!(
			0,
			app.world()
				.iter_entities()
				.filter(|e| e.contains::<_Child>())
		);
	}

	#[test]
	fn do_nothing_if_not_pressed() {
		let mut app = setup();
		let entities = [
			app.world_mut()
				.spawn((
					KeyBind(Input {
						action: _Action,
						input: _Input,
					}),
					Interaction::None,
				))
				.with_children(_Child::spawn_nested_once)
				.id(),
			app.world_mut()
				.spawn((
					KeyBind(Input {
						action: _Action,
						input: _Input,
					}),
					Interaction::Hovered,
				))
				.with_children(_Child::spawn_nested_once)
				.id(),
		];

		app.update();

		assert_eq!(
			([true, true], [false, false], 4),
			(
				app.world()
					.entity(entities)
					.map(|e| e.contains::<KeyBind<Input<_Action, _Input>>>()),
				app.world()
					.entity(entities)
					.map(|e| e.contains::<KeyBind<Rebinding<_Action, _Input>>>()),
				app.world()
					.iter_entities()
					.filter(|e| e.contains::<_Child>())
					.count()
			)
		);
	}
}
