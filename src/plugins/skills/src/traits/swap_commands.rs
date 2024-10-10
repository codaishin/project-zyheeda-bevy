use crate::items::{inventory_key::InventoryKey, slot_key::SlotKey, Item};
use common::{
	components::{Collection, Swap},
	traits::{
		get::GetMut,
		swap_command::{SwapCommands, SwapError, SwapIn, SwapResult, SwappedOut},
	},
};
use std::marker::PhantomData;

trait Keys<T1, T2> {
	fn keys(&self) -> (T1, T2);
}

impl Keys<InventoryKey, SlotKey> for Swap<InventoryKey, SlotKey> {
	fn keys(&self) -> (InventoryKey, SlotKey) {
		(self.0, self.1)
	}
}

impl Keys<InventoryKey, SlotKey> for Swap<SlotKey, InventoryKey> {
	fn keys(&self) -> (InventoryKey, SlotKey) {
		(self.1, self.0)
	}
}

pub struct SwapController<'a, TInnerKey, TOuterKey, TContainer, TSwaps> {
	pub container: &'a mut TContainer,
	pub swaps: &'a mut TSwaps,
	phantom_data: PhantomData<(TInnerKey, TOuterKey)>,
}

impl<'a, TInnerKey, TOuterKey, TContainer, TSwaps>
	SwapController<'a, TInnerKey, TOuterKey, TContainer, TSwaps>
{
	pub fn new(container: &'a mut TContainer, swaps: &'a mut TSwaps) -> Self {
		Self {
			container,
			swaps,
			phantom_data: PhantomData,
		}
	}
}

struct RetryFailed<T>(T);

impl<'a, TContainer, TSwap, TContainerKey> SwapCommands<SlotKey, Item>
	for SwapController<'a, TContainerKey, SlotKey, TContainer, Collection<TSwap>>
where
	TContainer: GetMut<TContainerKey, Option<Item>>,
	TSwap: Keys<TContainerKey, SlotKey> + Clone,
{
	fn try_swap(&mut self, swap_fn: impl FnMut(SlotKey, SwapIn<Item>) -> SwapResult<Item>) {
		let Collection(swaps) = self.swaps;

		*swaps = swaps
			.iter()
			.filter_map(apply_swaps(self.container, swap_fn))
			.map(retry_failed)
			.collect();
	}

	fn is_empty(&self) -> bool {
		self.swaps.0.is_empty()
	}
}

fn apply_swaps<
	'a,
	TContainer: GetMut<TContainerKey, Option<Item>>,
	TContainerKey,
	TSwap: Keys<TContainerKey, SlotKey> + Clone,
>(
	container: &'a mut TContainer,
	mut swap_fn: impl FnMut(SlotKey, SwapIn<Item>) -> SwapResult<Item> + 'a,
) -> impl FnMut(&TSwap) -> Option<RetryFailed<TSwap>> + 'a {
	move |swap| {
		let (container_key, slot_key) = swap.keys();
		let item = container.get_mut(&container_key)?;

		match swap_fn(slot_key, SwapIn(item.clone())) {
			Ok(SwappedOut(new_item)) => {
				*item = new_item;
				None
			}
			Err(SwapError::Disregard) => None,
			Err(SwapError::TryAgain) => Some(RetryFailed(swap.clone())),
		}
	}
}

fn retry_failed<TSwap>(RetryFailed(swap): RetryFailed<TSwap>) -> TSwap {
	swap
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::{components::Side, traits::swap_command::SwapError};

	#[test]
	fn swap_inventory_slot_keys() {
		let swap = Swap(InventoryKey(42), SlotKey::BottomHand(Side::Right));

		assert_eq!(
			(InventoryKey(42), SlotKey::BottomHand(Side::Right)),
			swap.keys()
		);
	}

	#[test]
	fn swap_slot_inventory_keys() {
		let swap = Swap(SlotKey::BottomHand(Side::Right), InventoryKey(42));

		assert_eq!(
			(InventoryKey(42), SlotKey::BottomHand(Side::Right)),
			swap.keys()
		);
	}

	#[derive(Clone, Copy, Debug, PartialEq)]
	struct _InnerKey(usize);

	#[derive(Clone, Debug, PartialEq)]
	struct _Swap(_InnerKey, SlotKey);

	impl Keys<_InnerKey, SlotKey> for _Swap {
		fn keys(&self) -> (_InnerKey, SlotKey) {
			(self.0, self.1)
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Container(Vec<Option<Item>>);

	impl GetMut<_InnerKey, Option<Item>> for _Container {
		fn get_mut(&mut self, key: &_InnerKey) -> Option<&mut Option<Item>> {
			self.0.get_mut(key.0)
		}
	}

	#[test]
	fn set_swapped_out_item_in_inventory() {
		let mut container = _Container(vec![None]);
		let mut swaps = Collection::new([_Swap(_InnerKey(0), SlotKey::BottomHand(Side::Left))]);

		SwapController::new(&mut container, &mut swaps).try_swap(|_, _| {
			Ok(SwappedOut(Some(Item {
				name: "swapped out",
				..default()
			})))
		});

		assert_eq!(
			_Container(vec![Some(Item {
				name: "swapped out",
				..default()
			})]),
			container
		);
	}

	#[test]
	fn pass_swap_in_values_to_callback() {
		let mut container = _Container(vec![Some(Item {
			name: "swap in",
			..default()
		})]);
		let mut swaps = Collection::new([_Swap(_InnerKey(0), SlotKey::BottomHand(Side::Left))]);

		SwapController::new(&mut container, &mut swaps).try_swap(|slot_key, item| {
			assert_eq!(
				(
					SlotKey::BottomHand(Side::Left),
					SwapIn(Some(Item {
						name: "swap in",
						..default()
					}))
				),
				(slot_key, item)
			);
			Ok(SwappedOut(Some(Item::default())))
		});
	}

	#[test]
	fn clear_swaps() {
		let mut container = _Container(vec![Some(Item {
			name: "unaffected",
			..default()
		})]);
		let mut swaps = Collection::new([_Swap(_InnerKey(0), SlotKey::BottomHand(Side::Left))]);

		SwapController::new(&mut container, &mut swaps)
			.try_swap(|_, _| Ok(SwappedOut(Some(Item::default()))));

		assert_eq!(Collection::new([]), swaps);
	}

	#[test]
	fn retain_swap_try_again_errors() {
		let mut container = _Container(vec![
			Some(Item {
				name: "disregard error",
				..default()
			}),
			Some(Item {
				name: "try again error",
				..default()
			}),
			Some(Item::default()),
		]);
		let mut swaps = Collection::new([
			_Swap(_InnerKey(0), SlotKey::default()),
			_Swap(_InnerKey(1), SlotKey::default()),
			_Swap(_InnerKey(2), SlotKey::default()),
		]);

		SwapController::new(&mut container, &mut swaps).try_swap(|_, SwapIn(item)| match item {
			Some(item) if item.name == "disregard error" => Err(SwapError::Disregard),
			Some(item) if item.name == "try again error" => Err(SwapError::TryAgain),
			_ => Ok(SwappedOut(default())),
		});

		assert_eq!(
			Collection::new([_Swap(_InnerKey(1), SlotKey::BottomHand(Side::Right))]),
			swaps
		);
	}

	#[test]
	fn swaps_not_empty() {
		let mut container = _Container(vec![]);
		let mut swaps = Collection::new([_Swap(_InnerKey(0), SlotKey::BottomHand(Side::Left))]);

		assert!(!SwapController::new(&mut container, &mut swaps).is_empty());
	}

	#[test]
	fn swaps_empty() {
		let mut container = _Container(vec![]);
		let mut swaps = Collection::<_Swap>::new([]);

		assert!(SwapController::new(&mut container, &mut swaps).is_empty());
	}
}
