use crate::{Input, KeyBind, Rebinding, components::button_state::UiButtonState};
use bevy::prelude::*;
use common::prelude::*;

impl<TAction> KeyBind<Input<TAction>>
where
	TAction: Copy + ThreadSafe,
{
	pub(crate) fn rebind_on_release(
		mut commands: ZyheedaCommands,
		key_binds: Query<KeyBinds<TAction>>,
	) {
		for (entity, KeyBind(input), UiButtonState(state), children) in &key_binds {
			if !matches!(state, InputState::Released { just_now: true }) {
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

type KeyBinds<'a, TAction> = (
	Entity,
	&'a KeyBind<Input<TAction>>,
	&'a UiButtonState,
	Option<&'a Children>,
);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Rebinding;
	use test_case::test_case;
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

		app.add_systems(Update, KeyBind::<Input<_Action>>::rebind_on_release);

		app
	}

	#[test]
	fn insert_rebind_on_release_to() {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		let entity = app
			.world_mut()
			.spawn((
				KeyBind(Input {
					action: _Action,
					input: UserInput::MouseButton(MouseButton::Left),
				}),
				UiButtonState(InputState::just_released()),
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
				UiButtonState(InputState::just_released()),
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
				UiButtonState(InputState::just_released()),
			))
			.with_children(_Child::spawn_nested_once);

		app.update();

		let mut children = app.world_mut().query_filtered::<(), With<_Child>>();
		assert_count!(0, children.iter(app.world()));
	}

	#[test_case(UiButtonState(InputState::pressed()); "pressed")]
	#[test_case(UiButtonState(InputState::just_pressed()); "just pressed")]
	#[test_case(UiButtonState(InputState::released()); "released")]
	fn do_nothing_if(state: UiButtonState) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				KeyBind(Input {
					action: _Action,
					input: UserInput::MouseButton(MouseButton::Left),
				}),
				state,
			))
			.with_children(_Child::spawn_nested_once)
			.id();

		app.update();

		let mut children = app.world_mut().query_filtered::<(), With<_Child>>();
		assert_eq!(
			(true, false, 2),
			(
				app.world()
					.entity(entity)
					.contains::<KeyBind<Input<_Action>>>(),
				app.world()
					.entity(entity)
					.contains::<KeyBind<Rebinding<_Action>>>(),
				children.iter(app.world()).count()
			)
		);
	}
}
