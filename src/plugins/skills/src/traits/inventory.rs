use crate::{
	components::inventory::Inventory,
	items::{inventory_key::InventoryKey, slot_key::SlotKey, Item},
	skills::Skill,
};
use bevy::asset::Handle;
use common::{
	components::{Collection, Swap},
	traits::{
		accessor::Accessor,
		swap_command::{SwapCommands, SwapError, SwapIn, SwapResult, SwappedOut},
	},
};

trait Keys {
	fn keys(&self) -> (InventoryKey, SlotKey);
}

impl Keys for Swap<InventoryKey, SlotKey> {
	fn keys(&self) -> (InventoryKey, SlotKey) {
		(self.0, self.1)
	}
}

impl Keys for Swap<SlotKey, InventoryKey> {
	fn keys(&self) -> (InventoryKey, SlotKey) {
		(self.1, self.0)
	}
}

struct RetryFailed<T>(T);

impl<'a, TSkill, TSwap> SwapCommands<SlotKey, Item<TSkill>>
	for (&'a mut Inventory<TSkill>, &'a mut Collection<TSwap>)
where
	TSkill: Clone,
	TSwap: Keys + Clone,
{
	fn try_swap(
		&mut self,
		swap_fn: impl FnMut(SlotKey, SwapIn<Item<TSkill>>) -> SwapResult<Item<TSkill>>,
	) {
		let (Collection(items), Collection(swaps)) = self;

		*swaps = swaps
			.iter()
			.filter_map(apply_swaps(items, swap_fn))
			.map(retry_failed)
			.collect();
	}

	fn is_empty(&self) -> bool {
		let (.., Collection(swaps)) = self;
		swaps.is_empty()
	}
}

fn apply_swaps<'a, TSkill: Clone, TSwap: Keys + Clone>(
	items: &'a mut Vec<Option<Item<TSkill>>>,
	mut swap_fn: impl FnMut(SlotKey, SwapIn<Item<TSkill>>) -> SwapResult<Item<TSkill>> + 'a,
) -> impl FnMut(&TSwap) -> Option<RetryFailed<TSwap>> + 'a {
	move |swap| {
		let (InventoryKey(item_key), slot_key) = swap.keys();
		let item = items.get(item_key).cloned().flatten();
		match swap_fn(slot_key, SwapIn(item)) {
			Ok(SwappedOut(item)) => insert(items, item_key, item),
			Err(SwapError::Disregard) => None,
			Err(SwapError::TryAgain) => Some(RetryFailed(swap.clone())),
		}
	}
}

fn retry_failed<TSwap>(RetryFailed(swap): RetryFailed<TSwap>) -> TSwap {
	swap
}

fn insert<TSkill: Clone, TSwap>(
	inventory: &mut Vec<Option<Item<TSkill>>>,
	inventory_key: usize,
	item: Option<Item<TSkill>>,
) -> Option<TSwap> {
	if inventory.len() <= inventory_key {
		fill(inventory, inventory_key);
	}
	inventory[inventory_key] = item;
	None
}

impl Accessor<Inventory<Handle<Skill>>, (SlotKey, Option<Item<Handle<Skill>>>), Item<Handle<Skill>>>
	for Swap<InventoryKey, SlotKey>
{
	fn get_key_and_item(
		&self,
		inventory: &Inventory<Handle<Skill>>,
	) -> (SlotKey, Option<Item<Handle<Skill>>>) {
		let inventory = &inventory.0;
		let Swap(inventory_key, slot_key) = *self;

		if inventory_key.0 >= inventory.len() {
			return (slot_key, None);
		}
		(slot_key, inventory[inventory_key.0].clone())
	}

	fn with_item(
		&self,
		item: Option<Item<Handle<Skill>>>,
		inventory: &mut Inventory<Handle<Skill>>,
	) -> Self {
		let inventory = &mut inventory.0;
		let Swap(inventory_key, slot_key) = self;

		if inventory_key.0 >= inventory.len() {
			fill(inventory, inventory_key.0);
		}
		inventory[inventory_key.0] = item;
		Self(*inventory_key, *slot_key)
	}
}

impl Accessor<Inventory<Handle<Skill>>, (SlotKey, Option<Item<Handle<Skill>>>), Item<Handle<Skill>>>
	for Swap<SlotKey, InventoryKey>
{
	fn get_key_and_item(
		&self,
		inventory: &Inventory<Handle<Skill>>,
	) -> (SlotKey, Option<Item<Handle<Skill>>>) {
		Swap(self.1, self.0).get_key_and_item(inventory)
	}

	fn with_item(
		&self,
		item: Option<Item<Handle<Skill>>>,
		inventory: &mut Inventory<Handle<Skill>>,
	) -> Self {
		let Swap(inventory_key, slot_key) = Swap(self.1, self.0).with_item(item, inventory);
		Self(slot_key, inventory_key)
	}
}

fn fill<T: Clone>(inventory: &mut Vec<Option<Item<T>>>, inventory_key: usize) {
	let fill_len = inventory_key - inventory.len() + 1;
	inventory.extend(vec![None; fill_len]);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::{components::Side, traits::swap_command::SwapError};

	#[test]
	fn swap_inventory_slot_keys() {
		let swap = Swap(InventoryKey(42), SlotKey::Hand(Side::Main));

		assert_eq!((InventoryKey(42), SlotKey::Hand(Side::Main)), swap.keys());
	}

	#[test]
	fn swap_slot_inventory_keys() {
		let swap = Swap(SlotKey::Hand(Side::Main), InventoryKey(42));

		assert_eq!((InventoryKey(42), SlotKey::Hand(Side::Main)), swap.keys());
	}

	#[derive(Clone, Debug, PartialEq)]
	struct _Swap(InventoryKey, SlotKey);

	impl Keys for _Swap {
		fn keys(&self) -> (InventoryKey, SlotKey) {
			(self.0, self.1)
		}
	}

	#[test]
	fn set_swapped_out_item_in_inventory() {
		let mut inventory = Inventory::<()>::new([Some(Item {
			name: "swap in",
			..default()
		})]);
		let mut swaps = Collection::new([_Swap(InventoryKey(0), SlotKey::Hand(Side::Off))]);

		(&mut inventory, &mut swaps).try_swap(|_, _| {
			Ok(SwappedOut(Some(Item {
				name: "swapped out",
				..default()
			})))
		});

		assert_eq!(
			Inventory::<()>::new([Some(Item {
				name: "swapped out",
				..default()
			})]),
			inventory
		);
	}

	#[test]
	fn pass_swap_in_values_to_callback() {
		let mut inventory = Inventory::<()>::new([Some(Item {
			name: "swap in",
			..default()
		})]);
		let mut swaps = Collection::new([_Swap(InventoryKey(0), SlotKey::Hand(Side::Off))]);

		(&mut inventory, &mut swaps).try_swap(|slot_key, item| {
			assert_eq!(
				(
					SlotKey::Hand(Side::Off),
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
	fn handle_inventory_index_out_of_range() {
		let mut inventory = Inventory::<()>::new([Some(Item {
			name: "unaffected",
			..default()
		})]);
		let mut swaps = Collection::new([_Swap(InventoryKey(3), SlotKey::Hand(Side::Off))]);

		(&mut inventory, &mut swaps).try_swap(|_, _| {
			Ok(SwappedOut(Some(Item {
				name: "swapped out",
				..default()
			})))
		});

		assert_eq!(
			Inventory::<()>::new([
				Some(Item {
					name: "unaffected",
					..default()
				}),
				None,
				None,
				Some(Item {
					name: "swapped out",
					..default()
				})
			]),
			inventory
		);
	}

	#[test]
	fn clear_swaps() {
		let mut inventory = Inventory::<()>::new([Some(Item {
			name: "unaffected",
			..default()
		})]);
		let mut swaps = Collection::new([_Swap(InventoryKey(0), SlotKey::Hand(Side::Off))]);

		(&mut inventory, &mut swaps).try_swap(|_, _| Ok(SwappedOut(Some(Item::default()))));

		assert_eq!(Collection::new([]), swaps);
	}

	#[test]
	fn retain_swap_try_again_errors() {
		let mut inventory = Inventory::<()>::new([
			Some(Item {
				name: "disregard error",
				..default()
			}),
			Some(Item {
				name: "try again error",
				..default()
			}),
		]);
		let mut swaps = Collection::new([
			_Swap(InventoryKey(0), SlotKey::default()),
			_Swap(InventoryKey(1), SlotKey::default()),
			_Swap(InventoryKey(2), SlotKey::default()),
		]);

		(&mut inventory, &mut swaps).try_swap(|_, SwapIn(item)| match item {
			Some(item) if item.name == "disregard error" => Err(SwapError::Disregard),
			Some(item) if item.name == "try again error" => Err(SwapError::TryAgain),
			_ => Ok(SwappedOut(default())),
		});

		assert_eq!(
			Collection::new([_Swap(InventoryKey(1), SlotKey::Hand(Side::Main))]),
			swaps
		);
	}

	#[test]
	fn swaps_not_empty() {
		let mut inventory = Inventory::<()>::new([]);
		let mut swaps = Collection::new([_Swap(InventoryKey(0), SlotKey::Hand(Side::Off))]);

		assert!(!(&mut inventory, &mut swaps).is_empty());
	}

	#[test]
	fn swaps_empty() {
		let mut inventory = Inventory::<()>::new([]);
		let mut swaps = Collection::<_Swap>::new([]);

		assert!((&mut inventory, &mut swaps).is_empty());
	}
}

#[cfg(test)]
mod test_swap_inventory_key_slot_key {
	use super::*;
	use bevy::prelude::default;
	use common::components::Side;

	#[test]
	fn get_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(InventoryKey(0), slot_key);

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_second_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my second item",
			..default()
		};
		let inventory = Inventory::new([None, Some(item.clone())]);
		let swap = Swap(InventoryKey(1), slot_key);

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_slot_key() {
		let slot_key = SlotKey::Hand(Side::Main);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(InventoryKey(0), slot_key);

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_out_of_range() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item)]);
		let swap = Swap(InventoryKey(42), slot_key);

		assert_eq!((slot_key, None), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn copy_with_item() {
		let orig = Item {
			name: "orig",
			..default()
		};
		let replace = Item {
			name: "replace",
			..default()
		};
		let mut inventory = Inventory::new([Some(orig)]);
		let swap = Swap(InventoryKey(0), SlotKey::Hand(Side::Off));
		let copy = swap.with_item(Some(replace.clone()), &mut inventory);

		assert_eq!((swap, vec![Some(replace)]), (copy, inventory.0));
	}

	#[test]
	fn copy_with_item_push_back() {
		let item = Item {
			name: "my item",
			..default()
		};
		let new = Item {
			name: "new item",
			..default()
		};
		let mut inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(InventoryKey(1), SlotKey::Hand(Side::Off));
		let copy = swap.with_item(Some(new.clone()), &mut inventory);

		assert_eq!((swap, vec![Some(item), Some(new)]), (copy, inventory.0));
	}

	#[test]
	fn copy_with_item_out_of_range() {
		let item = Item {
			name: "my item",
			..default()
		};
		let new = Item {
			name: "new item",
			..default()
		};
		let mut inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(InventoryKey(3), SlotKey::Hand(Side::Off));
		let copy = swap.with_item(Some(new.clone()), &mut inventory);

		assert_eq!(
			(swap, vec![Some(item), None, None, Some(new)]),
			(copy, inventory.0)
		);
	}
}

#[cfg(test)]
mod test_swap_slot_key_inventory_key {
	use super::*;
	use bevy::prelude::default;
	use common::components::Side;

	#[test]
	fn get_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(slot_key, InventoryKey(0));

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_second_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my second item",
			..default()
		};
		let inventory = Inventory::new([None, Some(item.clone())]);
		let swap = Swap(slot_key, InventoryKey(1));

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_slot_key() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(slot_key, InventoryKey(0));

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_out_of_range() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(slot_key, InventoryKey(42));

		assert_eq!((slot_key, None), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn copy_with_item() {
		let orig = Item {
			name: "orig",
			..default()
		};
		let replace = Item {
			name: "replace",
			..default()
		};
		let mut inventory = Inventory::new([Some(orig)]);
		let swap = Swap(SlotKey::Hand(Side::Off), InventoryKey(0));
		let copy = swap.with_item(Some(replace.clone()), &mut inventory);

		assert_eq!((swap, vec![Some(replace)]), (copy, inventory.0));
	}

	#[test]
	fn copy_with_item_push_back() {
		let item = Item {
			name: "my item",
			..default()
		};
		let new = Item {
			name: "new item",
			..default()
		};
		let mut inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(SlotKey::Hand(Side::Off), InventoryKey(1));
		let copy = swap.with_item(Some(new.clone()), &mut inventory);

		assert_eq!((swap, vec![Some(item), Some(new)]), (copy, inventory.0));
	}

	#[test]
	fn copy_with_item_out_of_range() {
		let item = Item {
			name: "my item",
			..default()
		};
		let new = Item {
			name: "new item",
			..default()
		};
		let mut inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(SlotKey::Hand(Side::Off), InventoryKey(3));
		let copy = swap.with_item(Some(new.clone()), &mut inventory);

		assert_eq!(
			(swap, vec![Some(item), None, None, Some(new)]),
			(copy, inventory.0)
		);
	}
}
