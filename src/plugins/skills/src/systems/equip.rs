use crate::{components::slots::Slots, item::Item, traits::swap_commands::SwapController};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	tools::slot_key::SlotKey,
	traits::{
		swap_command::{SwapCommands, SwapError, SwapIn, SwappedOut},
		try_remove_from::TryRemoveFrom,
	},
};
use std::mem::swap;

type Components<'a, TContainer, TSwaps> =
	(Entity, &'a mut Slots, &'a mut TContainer, &'a mut TSwaps);

pub fn equip_item<TContainer, TInnerKey, TSwaps>(
	mut commands: Commands,
	mut agent: Query<Components<TContainer, TSwaps>>,
) -> Vec<Result<(), Error>>
where
	TContainer: Component,
	TSwaps: Component,
	for<'a> SwapController<'a, TInnerKey, SlotKey, TContainer, TSwaps>:
		SwapCommands<SlotKey, Handle<Item>>,
{
	let mut results = vec![];
	let commands = &mut commands;

	for (agent, mut slots, mut container, mut swaps) in &mut agent {
		let slots = slots.as_mut();
		let mut swap_controller = SwapController::new(container.as_mut(), swaps.as_mut());

		swap_controller.try_swap(|slot_key, SwapIn(item)| {
			match try_swap(slots, slot_key, item.clone()) {
				Ok(swapped_out) => Ok(swapped_out),
				Err((swap_error, log_error)) => {
					results.push(Err(log_error));
					Err(swap_error)
				}
			}
		});

		if swap_controller.is_empty() {
			commands.try_remove_from::<TSwaps>(agent);
		}
	}

	results
}

fn try_swap(
	slots: &mut Slots,
	slot_key: SlotKey,
	item: Option<Handle<Item>>,
) -> Result<SwappedOut<Handle<Item>>, (SwapError, Error)> {
	let slot = get_slot(slots, slot_key)?;

	Ok(swap_item(item, slot))
}

fn get_slot(
	slots: &mut Slots,
	slot_key: SlotKey,
) -> Result<&mut Option<Handle<Item>>, (SwapError, Error)> {
	match slots.0.get_mut(&slot_key) {
		Some(slot) => Ok(slot),
		None => Err((SwapError::TryAgain, slot_warning(slot_key))),
	}
}

fn swap_item(
	mut item: Option<Handle<Item>>,
	slot: &mut Option<Handle<Item>>,
) -> SwappedOut<Handle<Item>> {
	swap(&mut item, slot);

	SwappedOut(item)
}

fn slot_warning(slot: SlotKey) -> Error {
	Error {
		msg: format!("Slot `{:?}` not found, retrying next update", slot),
		lvl: Level::Warning,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		systems::log::test_tools::{fake_log_error_many_recourse, FakeErrorLogManyResource},
		test_tools::utils::{new_handle, SingleThreadedApp},
		tools::slot_key::Side,
		traits::swap_command::{SwapError, SwapIn, SwapResult, SwappedOut},
	};
	use std::collections::HashMap;

	#[derive(Component, Default, PartialEq, Debug)]
	struct _Swaps {
		is_empty: bool,
	}

	#[derive(Component, PartialEq, Clone, Debug, Default)]
	pub struct _Container {
		swap_ins: HashMap<SlotKey, SwapIn<Handle<Item>>>,
		swap_outs: HashMap<SlotKey, SwappedOut<Handle<Item>>>,
		errors: HashMap<SlotKey, SwapError>,
	}

	impl SwapCommands<SlotKey, Handle<Item>> for SwapController<'_, (), SlotKey, _Container, _Swaps> {
		fn try_swap(
			&mut self,
			mut swap_fn: impl FnMut(SlotKey, SwapIn<Handle<Item>>) -> SwapResult<Handle<Item>>,
		) {
			let SwapController { container, .. } = self;
			for (slot_key, swap_in) in container.swap_ins.clone() {
				match swap_fn(slot_key, swap_in.clone()) {
					Ok(swap_out) => {
						container.swap_outs.insert(slot_key, swap_out);
					}
					Err(error) => {
						container.errors.insert(slot_key, error);
					}
				}
			}
		}

		fn is_empty(&self) -> bool {
			let SwapController { swaps, .. } = self;
			swaps.is_empty
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			equip_item::<_Container, (), _Swaps>.pipe(fake_log_error_many_recourse),
		);

		app
	}

	#[test]
	fn set_swap_in_item() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Slots::new([(SlotKey::BottomHand(Side::Right), None)]),
				_Container {
					swap_ins: HashMap::from([(
						SlotKey::BottomHand(Side::Right),
						SwapIn(Some(item.clone())),
					)]),
					..default()
				},
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Slots::new([(
				SlotKey::BottomHand(Side::Right),
				Some(item),
			)])),
			agent.get::<Slots>()
		);
	}

	#[test]
	fn set_swap_out_item() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Slots::new([(SlotKey::BottomHand(Side::Right), Some(item.clone()))]),
				_Container {
					swap_ins: HashMap::from([(SlotKey::BottomHand(Side::Right), SwapIn(None))]),
					..default()
				},
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);
		let container = agent.get::<_Container>().unwrap();

		assert_eq!(
			HashMap::from([(SlotKey::BottomHand(Side::Right), SwappedOut(Some(item)),)]),
			container.swap_outs
		);
	}

	#[test]
	fn try_again_error_when_slot_not_found() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Slots::new([(SlotKey::BottomHand(Side::Right), None)]),
				_Container {
					swap_ins: HashMap::from([(SlotKey::BottomHand(Side::Left), SwapIn(None))]),
					..default()
				},
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);
		let container = agent.get::<_Container>().unwrap();

		assert_eq!(
			HashMap::from([(SlotKey::BottomHand(Side::Left), SwapError::TryAgain)]),
			container.errors
		);
	}

	#[test]
	fn return_error_when_slot_not_found() {
		let mut app = setup();
		app.world_mut().spawn((
			Slots::new([(SlotKey::BottomHand(Side::Right), None)]),
			_Container {
				swap_ins: HashMap::from([(SlotKey::BottomHand(Side::Left), SwapIn(None))]),
				..default()
			},
			_Swaps { is_empty: false },
		));

		app.update();

		let error_log = app.world().get_resource::<FakeErrorLogManyResource>();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![slot_warning(
				SlotKey::BottomHand(Side::Left)
			)])),
			error_log
		);
	}

	#[test]
	fn remove_swap_component_when_empty() {
		let mut app = setup();

		let agent = app
			.world_mut()
			.spawn((
				Slots::default(),
				_Container::default(),
				_Swaps { is_empty: true },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Swaps>());
	}

	#[test]
	fn do_not_remove_swap_component_when_not_empty() {
		let mut app = setup();

		let agent = app
			.world_mut()
			.spawn((
				Slots::default(),
				_Container::default(),
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Swaps { is_empty: false }), agent.get::<_Swaps>());
	}
}
