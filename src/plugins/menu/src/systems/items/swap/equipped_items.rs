use bevy::prelude::*;
use common::{
	components::{Collection, Swap},
	errors::{Error, Level},
	tools::slot_key::SlotKey,
	traits::{
		handles_equipment::{ItemAsset, KeyOutOfBounds, WriteItem},
		try_remove_from::TryRemoveFrom,
	},
};

#[allow(clippy::type_complexity)]
pub fn swap_equipped_items<TSlots>(
	mut commands: Commands,
	mut slots_to_swap: Query<(Entity, &mut TSlots, &Collection<Swap<SlotKey, SlotKey>>)>,
) -> Vec<Result<(), Error>>
where
	TSlots:
		Component + ItemAsset<TKey = SlotKey> + WriteItem<SlotKey, Option<Handle<TSlots::TItem>>>,
{
	let mut results = vec![];

	for (agent, mut slots, swaps) in &mut slots_to_swap {
		for swap in &swaps.0 {
			let [result0, result1] = do_swap(swap, &mut slots);
			if result0.is_err() {
				results.push(result0);
			}
			if result1.is_err() {
				results.push(result1);
			}
		}

		commands.try_remove_from::<Collection<Swap<SlotKey, SlotKey>>>(agent);
	}

	results
}

fn do_swap<TSlots>(swap: &Swap<SlotKey, SlotKey>, slots: &mut Mut<TSlots>) -> [Result<(), Error>; 2]
where
	TSlots: ItemAsset<TKey = SlotKey> + WriteItem<SlotKey, Option<Handle<TSlots::TItem>>>,
{
	let slot_results = [
		slots.item_asset(&swap.0).cloned().map_err(no_slot(swap.0)),
		slots.item_asset(&swap.1).cloned().map_err(no_slot(swap.1)),
	];

	let [Ok(slot0), Ok(slot1)] = slot_results else {
		return slot_results.map(drop_ok);
	};

	slots.write_item(&swap.0, slot1);
	slots.write_item(&swap.1, slot0);

	[Ok(()), Ok(())]
}

fn drop_ok<V>(result: Result<V, Error>) -> Result<(), Error> {
	match result {
		Err(error) => Err(error),
		Ok(_) => Ok(()),
	}
}

fn no_slot(slot_key: SlotKey) -> impl Fn(KeyOutOfBounds) -> Error {
	move |_| Error {
		msg: format!("{:?}: Slot not found", slot_key),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{test_tools::utils::new_handle, tools::slot_key::Side};
	use std::collections::HashMap;

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Skill(&'static str);

	#[derive(Component, Debug, PartialEq)]
	struct _Slots(HashMap<SlotKey, Option<Handle<_Skill>>>);

	impl _Slots {
		fn new<const N: usize>(skills: [(SlotKey, Option<Handle<_Skill>>); N]) -> Self {
			Self(HashMap::from(skills))
		}
	}

	impl ItemAsset for _Slots {
		type TKey = SlotKey;
		type TItem = _Skill;

		fn item_asset(
			&self,
			key: &Self::TKey,
		) -> Result<&Option<Handle<Self::TItem>>, KeyOutOfBounds> {
			let Some(item) = self.0.get(key) else {
				return Err(KeyOutOfBounds);
			};

			Ok(item)
		}
	}

	impl WriteItem<SlotKey, Option<Handle<_Skill>>> for _Slots {
		fn write_item(&mut self, key: &SlotKey, value: Option<Handle<_Skill>>) {
			self.0.insert(*key, value);
		}
	}

	#[derive(Clone)]
	struct _Mount {
		entity: Entity,
		handle: Handle<Scene>,
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn swap_items() -> Result<(), RunSystemError> {
		let left_item = new_handle();
		let right_item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Slots::new([
					(SlotKey::BottomHand(Side::Left), Some(left_item.clone())),
					(SlotKey::BottomHand(Side::Right), Some(right_item.clone())),
				]),
				Collection(
					[Swap(
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Right),
					)]
					.into(),
				),
			))
			.id();

		let errors = app
			.world_mut()
			.run_system_once(swap_equipped_items::<_Slots>)?;

		let slots = app.world().entity(agent).get::<_Slots>();
		assert_eq!(
			(
				Some(&_Slots::new([
					(SlotKey::BottomHand(Side::Left), Some(right_item.clone())),
					(SlotKey::BottomHand(Side::Right), Some(left_item.clone())),
				])),
				vec![]
			),
			(slots, errors)
		);
		Ok(())
	}

	#[test]
	fn remove_collection() -> Result<(), RunSystemError> {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Slots([].into()),
				Collection::<Swap<SlotKey, SlotKey>>([].into()),
			))
			.id();

		app.world_mut()
			.run_system_once(swap_equipped_items::<_Slots>)?;

		let agent = app.world().entity(agent);
		assert!(!agent.contains::<Collection<Swap<SlotKey, SlotKey>>>());
		Ok(())
	}

	#[test]
	fn log_slot_errors() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			_Slots([].into()),
			Collection(
				[Swap(
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				)]
				.into(),
			),
		));

		let errors = app
			.world_mut()
			.run_system_once(swap_equipped_items::<_Slots>)?;

		assert_eq!(
			vec![
				Err(no_slot(SlotKey::BottomHand(Side::Left))(KeyOutOfBounds)),
				Err(no_slot(SlotKey::BottomHand(Side::Right))(KeyOutOfBounds)),
			],
			errors
		);
		Ok(())
	}
}
