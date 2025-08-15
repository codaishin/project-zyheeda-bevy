use crate::{events::skill::SkillEvent, systems::get_inputs::Input, traits::IterHolding};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::{PlayerSlot, SlotKey},
	traits::accessors::get::{RefAs, RefInto},
};

impl SkillEvent {
	pub(crate) fn trigger_from_input_for<TPlayer, TQueue>(
		In(input): In<Input>,
		players: Query<(Entity, &TQueue), With<TPlayer>>,
		mut events: EventWriter<Self>,
	) where
		TPlayer: Component,
		TQueue: Component + IterHolding,
		TQueue::TItem: for<'a> RefInto<'a, SlotKey>,
	{
		for (entity, queue) in &players {
			for key in &input.just_pressed {
				events.write(SkillEvent::Hold {
					agent: entity,
					key: SlotKey::from(*key),
				});
			}

			for item in queue.iter_holding() {
				let key = item.ref_as::<SlotKey>();

				if matches!(PlayerSlot::try_from(key), Ok(key) if input.pressed.contains(&key)) {
					continue;
				}

				events.write(SkillEvent::Release { agent: entity, key });
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::action_key::slot::{PlayerSlot, Side, SlotKey};
	use testing::{SingleThreadedApp, get_current_update_events};

	#[derive(Component)]
	struct _Player;

	#[derive(Component)]
	struct _Queue {
		holding: Vec<_Item>,
	}

	impl<T> From<T> for _Queue
	where
		T: IntoIterator<Item = _Item>,
	{
		fn from(items: T) -> Self {
			Self {
				holding: Vec::from_iter(items),
			}
		}
	}

	impl IterHolding for _Queue {
		type TItem = _Item;

		fn iter_holding(&self) -> impl Iterator<Item = &'_ Self::TItem> {
			self.holding.iter()
		}
	}

	struct _Item(SlotKey);

	impl From<&_Item> for SlotKey {
		fn from(_Item(key): &_Item) -> Self {
			*key
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_event::<SkillEvent>();

		app
	}

	#[test]
	fn trigger_hold() -> Result<(), RunSystemError> {
		let mut app = setup();
		let input = Input {
			just_pressed: vec![PlayerSlot::Upper(Side::Left)],
			..default()
		};
		let players = [
			app.world_mut().spawn((_Player, _Queue::from([]))).id(),
			app.world_mut().spawn((_Player, _Queue::from([]))).id(),
		];

		app.world_mut()
			.run_system_once_with(SkillEvent::trigger_from_input_for::<_Player, _Queue>, input)?;

		assert_eq!(
			players
				.into_iter()
				.map(|entity| SkillEvent::Hold {
					agent: entity,
					key: SlotKey::from(PlayerSlot::Upper(Side::Left))
				})
				.collect::<Vec<_>>(),
			get_current_update_events!(app, SkillEvent, |e| *e).collect::<Vec<_>>()
		);
		Ok(())
	}

	#[test]
	fn do_not_trigger_hold_for_non_player_entities() -> Result<(), RunSystemError> {
		let mut app = setup();
		let input = Input {
			just_pressed: vec![PlayerSlot::Upper(Side::Left)],
			..default()
		};
		app.world_mut().spawn(_Queue::from([]));
		app.world_mut().spawn(_Queue::from([]));

		app.world_mut()
			.run_system_once_with(SkillEvent::trigger_from_input_for::<_Player, _Queue>, input)?;

		assert_eq!(
			None,
			get_current_update_events!(app, SkillEvent, |e| *e).next()
		);
		Ok(())
	}

	#[test]
	fn trigger_release_when_hold_item_not_contained_in_pressed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let input = Input::default();
		let player_with_holding = app
			.world_mut()
			.spawn((
				_Player,
				_Queue::from([_Item(SlotKey::from(PlayerSlot::Lower(Side::Right)))]),
			))
			.id();
		app.world_mut().spawn((_Player, _Queue::from([])));

		app.world_mut()
			.run_system_once_with(SkillEvent::trigger_from_input_for::<_Player, _Queue>, input)?;

		assert_eq!(
			vec![SkillEvent::Release {
				agent: player_with_holding,
				key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
			}],
			get_current_update_events!(app, SkillEvent, |e| *e).collect::<Vec<_>>()
		);
		Ok(())
	}

	#[test]
	fn do_not_trigger_release_when_hold_item_contained_in_pressed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let input = Input {
			pressed: vec![PlayerSlot::Lower(Side::Right)],
			..default()
		};
		app.world_mut().spawn((
			_Player,
			_Queue::from([_Item(SlotKey::from(PlayerSlot::Lower(Side::Right)))]),
		));

		app.world_mut()
			.run_system_once_with(SkillEvent::trigger_from_input_for::<_Player, _Queue>, input)?;

		assert_eq!(
			None,
			get_current_update_events!(app, SkillEvent, |e| *e).next()
		);
		Ok(())
	}
}
