use crate::{Input, KeyBind, Rebinding};
use bevy::{prelude::*, ui_widgets::Activate};
use common::prelude::*;

impl<TAction> KeyBind<Input<TAction>>
where
	TAction: Copy + ThreadSafe,
{
	pub(crate) fn rebind(
		activated: On<Activate>,
		mut commands: ZyheedaCommands,
		key_binds: Query<(&Self, Option<&Children>)>,
	) {
		let Ok((KeyBind(input), children)) = key_binds.get(activated.entity) else {
			return;
		};

		commands.try_apply_on(&activated.entity, |mut e| {
			e.try_insert(KeyBind(Rebinding(*input)));
			e.try_remove::<Self>();
		});

		let Some(children) = children else {
			return;
		};

		for child in children {
			commands.try_apply_on(child, |e| e.try_despawn());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Rebinding;
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

		app.add_observer(KeyBind::<Input<_Action>>::rebind);

		app
	}

	#[test]
	fn insert_rebind() {
		let mut app = setup();
		let mut entity = app.world_mut().spawn(KeyBind(Input {
			action: _Action,
			input: UserInput::MouseButton(MouseButton::Left),
		}));

		let entity = entity.trigger(|entity| Activate { entity }).id();
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
		let mut entity = app.world_mut().spawn(KeyBind(Input {
			action: _Action,
			input: UserInput::MouseButton(MouseButton::Left),
		}));

		let entity = entity.trigger(|entity| Activate { entity }).id();
		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<KeyBind<Input<_Action>>>()
		);
	}

	#[test]
	fn remove_all_input_children() {
		let mut app = setup();
		let mut entity = app.world_mut().spawn(KeyBind(Input {
			action: _Action,
			input: UserInput::MouseButton(MouseButton::Left),
		}));
		entity.with_children(_Child::spawn_nested_once);

		entity.trigger(|entity| Activate { entity });
		app.update();

		let mut children = app.world_mut().query_filtered::<(), With<_Child>>();
		assert_count!(0, children.iter(app.world()));
	}
}
