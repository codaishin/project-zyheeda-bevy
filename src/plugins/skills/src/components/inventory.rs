use super::Item;
use crate::{items::inventory_key::InventoryKey, skills::Skill, traits::TryMap};
use bevy::asset::Handle;
use common::{components::Collection, traits::get::Get};

pub type Inventory<TSkill> = Collection<Option<Item<TSkill>>>;

impl Get<InventoryKey, Item<Handle<Skill>>> for Inventory<Handle<Skill>> {
	fn get(&self, key: &InventoryKey) -> Option<&Item<Handle<Skill>>> {
		let item = self.0.get(key.0)?;
		item.as_ref()
	}
}

impl<TIn, TOut> TryMap<TIn, TOut, Inventory<TOut>> for Inventory<TIn> {
	fn try_map(&self, mut map_fn: impl FnMut(&TIn) -> Option<TOut>) -> Inventory<TOut> {
		let inventory = self.0.iter().map(|item| {
			let item = item.as_ref()?;

			Some(Item {
				skill: item.skill.as_ref().and_then(&mut map_fn),
				name: item.name,
				model: item.model,
				item_type: item.item_type.clone(),
				mount: item.mount,
			})
		});

		Collection::<Option<Item<TOut>>>(inventory.collect())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::items::{ItemType, Mount};
	use bevy::utils::default;
	use std::collections::HashSet;

	#[test]
	fn get_first_item() {
		let inventory = Inventory::new([Some(Item {
			name: "my item",
			..default()
		})]);

		assert_eq!(
			Some(&Item {
				name: "my item",
				..default()
			}),
			inventory.get(&InventoryKey(0))
		);
	}

	#[test]
	fn get_none_when_empty() {
		let inventory = Inventory::new([]);

		assert_eq!(None, inventory.get(&InventoryKey(0)));
	}

	#[test]
	fn get_3rd_item() {
		let inventory = Inventory::new([
			None,
			None,
			Some(Item {
				name: "my item",
				..default()
			}),
		]);

		assert_eq!(
			Some(&Item {
				name: "my item",
				..default()
			}),
			inventory.get(&InventoryKey(2))
		);
	}

	struct _In(&'static str);

	#[derive(Debug, PartialEq)]
	struct _Out(&'static str);

	#[test]
	fn map_inventory_item_skills() {
		let inventory = Inventory::new([Some(Item {
			skill: Some(_In("my skill")),
			..default()
		})]);

		let inventory = inventory.try_map(|value| Some(_Out(value.0)));

		assert_eq!(
			Inventory::new([Some(Item {
				skill: Some(_Out("my skill")),
				..default()
			})]),
			inventory
		);
	}

	#[test]
	fn map_inventory_item_completely() {
		let inventory = Inventory::new([Some(Item {
			skill: Some(_In("my skill")),
			name: "my item",
			model: Some("model"),
			item_type: HashSet::from([ItemType::Bracer]),
			mount: Mount::Forearm,
		})]);

		let inventory = inventory.try_map(|value| Some(_Out(value.0)));

		assert_eq!(
			Inventory::new([Some(Item {
				skill: Some(_Out("my skill")),
				name: "my item",
				model: Some("model"),
				item_type: HashSet::from([ItemType::Bracer]),
				mount: Mount::Forearm,
			})]),
			inventory
		);
	}

	#[test]
	fn do_not_discard_empty_slots() {
		let inventory = Inventory::new([
			Some(Item {
				skill: Some(_In("my skill")),
				..default()
			}),
			None,
		]);

		let inventory = inventory.try_map(|value| Some(_Out(value.0)));

		assert_eq!(
			Inventory::new([
				Some(Item {
					skill: Some(_Out("my skill")),
					..default()
				}),
				None
			]),
			inventory
		);
	}

	#[test]
	fn do_not_discard_empty_skills() {
		let inventory = Inventory::<_In>::new([
			Some(Item {
				skill: None,
				..default()
			}),
			None,
		]);

		let inventory = inventory.try_map(|value| Some(_Out(value.0)));

		assert_eq!(
			Inventory::new([
				Some(Item {
					skill: None,
					..default()
				}),
				None
			]),
			inventory
		);
	}
}
