use crate::{Input, KeyBind, Rebinding};
use bevy::prelude::*;
use common::prelude::*;

impl<TAction> KeyBind<Input<TAction>>
where
	TAction: Copy + ThreadSafe,
{
	pub(crate) fn rebind_on_release(
		mut commands: ZyheedaCommands,
		key_binds: Query<KeyBinds<TAction>, Changed<Interaction>>,
	) {
		for (entity, KeyBind(input), current, last, children) in &key_binds {
			let Some(mut entity) = commands.get_mut(&entity) else {
				continue;
			};

			if !Self::just_released(&mut entity, last, current) {
				continue;
			}

			entity.try_insert(KeyBind(Rebinding(*input)));
			entity.try_remove::<Self>();

			let Some(children) = children else {
				continue;
			};

			for child in children {
				commands.try_apply_on(child, |e| e.try_despawn());
			}
		}
	}

	fn just_released(
		entity: &mut ZyheedaEntityCommands,
		last: Option<&LastInteraction>,
		current: &Interaction,
	) -> bool {
		entity.try_insert(LastInteraction(*current));

		matches!(
			(last, current),
			(
				Some(LastInteraction(Interaction::Pressed)),
				Interaction::None | Interaction::Hovered,
			)
		)
	}
}

type KeyBinds<'a, TAction> = (
	Entity,
	&'a KeyBind<Input<TAction>>,
	&'a Interaction,
	Option<&'a LastInteraction>,
	Option<&'a Children>,
);

#[derive(Component)]
pub(crate) struct LastInteraction(Interaction);

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

	#[test_case(Interaction::Hovered; "hovered")]
	#[test_case(Interaction::None; "no interaction")]
	fn insert_rebind_on_release_to(released: Interaction) {
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
		app.world_mut().entity_mut(entity).insert(released);
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

	#[test_case(Interaction::Hovered; "hovered")]
	#[test_case(Interaction::None; "none")]
	fn remove_input(released: Interaction) {
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
		app.world_mut().entity_mut(entity).insert(released);
		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<KeyBind<Input<_Action>>>(),
		);
	}

	#[test_case(Interaction::Hovered; "hovered")]
	#[test_case(Interaction::None; "none")]
	fn remove_all_input_children(released: Interaction) {
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
			.with_children(_Child::spawn_nested_once)
			.id();

		app.update();
		app.world_mut().entity_mut(entity).insert(released);
		app.update();

		let mut children = app.world_mut().query_filtered::<(), With<_Child>>();
		assert_count!(0, children.iter(app.world()));
	}

	#[test_case(Interaction::Hovered; "hovered")]
	#[test_case(Interaction::Pressed; "pressed")]
	#[test_case(Interaction::None; "no interaction")]
	fn do_nothing_if_just(state: Interaction) {
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
