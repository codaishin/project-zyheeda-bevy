use bevy::prelude::*;
use common::{
	components::{Collection, Swap},
	errors::{Error, Level},
	traits::try_remove_from::TryRemoveFrom,
};
use skills::{components::slots::Slots, slot_key::SlotKey};

type SlotsToSwap<'a> = (
	Entity,
	&'a mut Slots,
	&'a Collection<Swap<SlotKey, SlotKey>>,
);

pub fn swap_equipped_items(
	mut commands: Commands,
	mut slots_to_swap: Query<SlotsToSwap>,
) -> Vec<Result<(), Error>> {
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

fn do_swap(swap: &Swap<SlotKey, SlotKey>, slots: &mut Mut<Slots>) -> [Result<(), Error>; 2] {
	let slot_results = [
		slots.0.get(&swap.0).cloned().ok_or(no_slot(swap.0)),
		slots.0.get(&swap.1).cloned().ok_or(no_slot(swap.1)),
	];

	let [Ok(slot0), Ok(slot1)] = slot_results else {
		return slot_results.map(drop_ok);
	};

	if let Some(slot) = slots.0.get_mut(&swap.0) {
		*slot = slot1;
	}
	if let Some(slot) = slots.0.get_mut(&swap.1) {
		*slot = slot0;
	}

	[Ok(()), Ok(())]
}

fn drop_ok<V>(result: Result<V, Error>) -> Result<(), Error> {
	match result {
		Err(error) => Err(error),
		Ok(_) => Ok(()),
	}
}

fn no_slot(slot_key: SlotKey) -> Error {
	Error {
		msg: format!("{:?}: Slot not found", slot_key),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{components::Side, test_tools::utils::new_handle};

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
				Slots::new([
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

		let errors = app.world_mut().run_system_once(swap_equipped_items)?;

		let slots = app.world().entity(agent).get::<Slots>();
		assert_eq!(
			(
				Some(&Slots::new([
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
				Slots([].into()),
				Collection::<Swap<SlotKey, SlotKey>>([].into()),
			))
			.id();

		app.world_mut().run_system_once(swap_equipped_items)?;

		let agent = app.world().entity(agent);
		assert!(!agent.contains::<Collection<Swap<SlotKey, SlotKey>>>());
		Ok(())
	}

	#[test]
	fn log_slot_errors() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			Slots([].into()),
			Collection(
				[Swap(
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				)]
				.into(),
			),
		));

		let errors = app.world_mut().run_system_once(swap_equipped_items)?;

		assert_eq!(
			vec![
				Err(no_slot(SlotKey::BottomHand(Side::Left))),
				Err(no_slot(SlotKey::BottomHand(Side::Right))),
			],
			errors
		);
		Ok(())
	}
}
