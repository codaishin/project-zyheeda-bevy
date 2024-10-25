use crate::{
	inventory_key::InventoryKey,
	item::{SkillItem, SkillItemContent},
	traits::TryMap,
};
use common::{
	components::Collection,
	traits::accessors::get::{GetMut, GetRef},
};

pub type Inventory<TSkill> = Collection<Option<SkillItem<TSkill>>>;

impl<TSkill> GetRef<InventoryKey, SkillItem<TSkill>> for Inventory<TSkill> {
	fn get(&self, key: &InventoryKey) -> Option<&SkillItem<TSkill>> {
		let item = self.0.get(key.0)?;
		item.as_ref()
	}
}

impl<T> GetMut<InventoryKey, Option<SkillItem<T>>> for Inventory<T> {
	fn get_mut(&mut self, InventoryKey(index): &InventoryKey) -> Option<&mut Option<SkillItem<T>>> {
		let items = &mut self.0;

		if index >= &items.len() {
			fill(items, *index);
		}

		items.get_mut(*index)
	}
}

fn fill<T>(inventory: &mut Vec<Option<SkillItem<T>>>, inventory_key: usize) {
	let fill_len = inventory_key - inventory.len() + 1;
	for _ in 0..fill_len {
		inventory.push(None);
	}
}

impl<TIn, TOut> TryMap<TIn, TOut, Inventory<TOut>> for Inventory<TIn> {
	fn try_map(&self, mut map_fn: impl FnMut(&TIn) -> Option<TOut>) -> Inventory<TOut> {
		let inventory = self.0.iter().map(|item| {
			let item = item.as_ref()?;
			let map_fn = &mut map_fn;

			Some(SkillItem {
				content: SkillItemContent {
					render: item.content.render.clone(),
					skill: item.content.skill.as_ref().and_then(map_fn),
					item_type: item.content.item_type,
				},
				name: item.name,
			})
		});

		Collection::<Option<SkillItem<TOut>>>(inventory.collect())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	#[derive(Debug, PartialEq, Default)]
	struct _Item(&'static str);

	#[derive(Debug, PartialEq, Default)]
	struct _ItemType;

	#[test]
	fn get_first_item() {
		let inventory = Inventory::<_Item>::new([Some(SkillItem {
			name: "my item",
			..default()
		})]);

		assert_eq!(
			Some(&SkillItem {
				name: "my item",
				..default()
			}),
			inventory.get(&InventoryKey(0))
		);
	}

	#[test]
	fn get_none_when_empty() {
		let inventory = Inventory::<_Item>::new([]);

		assert_eq!(None, inventory.get(&InventoryKey(0)));
	}

	#[test]
	fn get_3rd_item() {
		let inventory = Inventory::<_Item>::new([
			None,
			None,
			Some(SkillItem {
				name: "my item",
				..default()
			}),
		]);

		assert_eq!(
			Some(&SkillItem {
				name: "my item",
				..default()
			}),
			inventory.get(&InventoryKey(2))
		);
	}

	#[derive(Debug, PartialEq, Default, Clone, Copy)]
	struct _In(&'static str);

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _InItemType;

	#[derive(Debug, PartialEq, Default)]
	struct _Out(&'static str);

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _OutItemType;

	impl From<_InItemType> for _OutItemType {
		fn from(_: _InItemType) -> Self {
			_OutItemType
		}
	}

	#[test]
	fn map_inventory_item_skills() {
		let inventory = Inventory::new([Some(SkillItem {
			content: SkillItemContent {
				skill: Some(_In("my skill")),
				..default()
			},
			..default()
		})]);

		let inventory = inventory.try_map(|_In(value)| Some(_Out(value)));

		assert_eq!(
			Inventory::new([Some(SkillItem {
				content: SkillItemContent {
					skill: Some(_Out("my skill")),
					..default()
				},
				..default()
			})]),
			inventory
		);
	}

	#[test]
	fn do_not_discard_empty_slots() {
		let inventory = Inventory::new([
			Some(SkillItem {
				content: SkillItemContent {
					skill: Some(_In("my skill")),
					..default()
				},
				..default()
			}),
			None,
		]);

		let inventory = inventory.try_map(|_In(value)| Some(_Out(value)));

		assert_eq!(
			Inventory::new([
				Some(SkillItem {
					content: SkillItemContent {
						skill: Some(_Out("my skill")),
						..default()
					},
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
			Some(SkillItem {
				content: SkillItemContent {
					skill: None,
					..default()
				},
				..default()
			}),
			None,
		]);

		let inventory = inventory.try_map(|_In(value)| Some(_Out(value)));

		assert_eq!(
			Inventory::new([
				Some(SkillItem {
					content: SkillItemContent {
						skill: None,
						..default()
					},
					..default()
				}),
				None
			]),
			inventory
		);
	}

	#[test]
	fn get_item_mut() {
		let mut inventory = Inventory::<_Item>::new([Some(SkillItem {
			name: "my item",
			..default()
		})]);

		let item = inventory.get_mut(&InventoryKey(0));
		assert_eq!(
			Some(&mut Some(SkillItem {
				name: "my item",
				..default()
			})),
			item
		);
	}

	#[test]
	fn get_item_mut_exceeding_range() {
		let mut inventory = Inventory::<_Item>::new([Some(SkillItem {
			name: "my item",
			..default()
		})]);

		*inventory.get_mut(&InventoryKey(1)).expect("no item found") = Some(SkillItem {
			name: "my other item",
			..default()
		});

		assert_eq!(
			Inventory::<_Item>::new([
				Some(SkillItem {
					name: "my item",
					..default()
				}),
				Some(SkillItem {
					name: "my other item",
					..default()
				})
			]),
			inventory
		);
	}
}
