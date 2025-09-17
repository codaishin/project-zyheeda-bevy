mod dto;

use crate::{
	components::{inventory::dto::InventoryDto, slots::Slots},
	item::{Item, ItemSkill, SkillItem},
	resources::skill_item_assets::SkillItemAssets,
};
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::SlotKey,
		inventory_key::InventoryKey,
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::GetParamEntry,
		handles_loadout::loadout::{LoadoutItem, LoadoutKey, SwapExternal, SwapInternal},
		iterate::Iterate,
	},
};
use macros::SavableComponent;
use std::iter::Enumerate;

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone)]
#[savable_component(dto = InventoryDto)]
pub struct Inventory(pub(crate) Vec<Option<Handle<Item>>>);

impl<T> From<T> for Inventory
where
	T: IntoIterator<Item = Option<Handle<Item>>>,
{
	fn from(items: T) -> Self {
		Self(Vec::from_iter(items))
	}
}

impl<'w, 's> GetParamEntry<'w, 's, InventoryKey> for Inventory {
	type TParam = SkillItemAssets<'w>;
	type TItem = SkillItem;

	fn get_param_entry(
		&self,
		InventoryKey(index): &InventoryKey,
		SkillItemAssets { items, skills }: &SkillItemAssets,
	) -> Option<Self::TItem> {
		let Self(inventory) = self;
		let item = inventory
			.get(*index)
			.and_then(|item| item.as_ref())
			.and_then(|item| items.get(item))?;
		let skill = item.skill.as_ref().and_then(|skill| skills.get(skill));

		Some(SkillItem {
			token: item.token.clone(),
			skill: skill.map(|skill| ItemSkill {
				token: skill.token.clone(),
				icon: skill.icon.clone(),
				execution: SkillExecution::None,
			}),
		})
	}
}

impl LoadoutKey for Inventory {
	type TKey = InventoryKey;
}

impl LoadoutItem for Inventory {
	type TItem = SkillItem;
}

impl SwapExternal<Slots> for Inventory {
	fn swap_external<TKey, TOtherKey>(&mut self, other: &mut Slots, a: TKey, b: TOtherKey)
	where
		TKey: Into<InventoryKey>,
		TOtherKey: Into<SlotKey>,
	{
		let InventoryKey(a) = a.into();
		if a >= self.0.len() {
			fill(&mut self.0, a);
		}
		let a = &mut self.0[a];
		let b = other.items.entry(b.into()).or_default();
		std::mem::swap(a, b);
	}
}

impl SwapInternal for Inventory {
	fn swap_internal<TKey>(&mut self, a: TKey, b: TKey)
	where
		TKey: Into<InventoryKey>,
	{
		let InventoryKey(a) = a.into();
		let InventoryKey(b) = b.into();
		let items = &mut self.0;
		let max = a.max(b);

		if max >= items.len() {
			fill(items, max);
		}

		items.swap(a, b);
	}
}

fn fill(inventory: &mut Vec<Option<Handle<Item>>>, inventory_key: usize) {
	let fill_len = inventory_key - inventory.len() + 1;
	for _ in 0..fill_len {
		inventory.push(None);
	}
}

impl<'a> Iterate<'a> for Inventory {
	type TItem = (InventoryKey, &'a Option<Handle<Item>>);
	type TIter = Iter<'a>;

	fn iterate(&'a self) -> Self::TIter {
		Iter {
			it: self.0.iter().enumerate(),
		}
	}
}

pub struct Iter<'a> {
	it: Enumerate<std::slice::Iter<'a, Option<Handle<Item>>>>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = (InventoryKey, &'a Option<Handle<Item>>);

	fn next(&mut self) -> Option<Self::Item> {
		let (i, item) = self.it.next()?;
		Some((InventoryKey(i), item))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{item::SkillItem, skills::Skill};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{tools::skill_execution::SkillExecution, traits::handles_localization::Token};
	use testing::{SingleThreadedApp, new_handle};

	fn setup<const I: usize, const S: usize>(
		items: [(&Handle<Item>, Item); I],
		skills: [(&Handle<Skill>, Skill); S],
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut item_assets = Assets::default();
		let mut skill_assets = Assets::default();

		for (id, asset) in items {
			item_assets.insert(id, asset);
		}

		for (id, asset) in skills {
			skill_assets.insert(id, asset);
		}

		app.insert_resource(item_assets);
		app.insert_resource(skill_assets);
		app
	}

	#[test]
	fn get_none_when_empty() -> Result<(), RunSystemError> {
		let mut app = setup([], []);

		app.world_mut()
			.run_system_once(|skill_items: SkillItemAssets| {
				let inventory = Inventory::from([]);

				assert_eq!(
					None,
					inventory.get_param_entry(&InventoryKey(0), &skill_items)
				);
			})
	}

	#[test]
	fn get_first_item() -> Result<(), RunSystemError> {
		let item_handle = new_handle();
		let skill_handle = new_handle();
		let icon_handle = new_handle();
		let mut app = setup(
			[(
				&item_handle,
				Item {
					token: Token::from("my item"),
					skill: Some(skill_handle.clone()),
					..default()
				},
			)],
			[(
				&skill_handle,
				Skill {
					token: Token::from("my skill"),
					icon: icon_handle.clone(),
					..default()
				},
			)],
		);

		app.world_mut()
			.run_system_once(move |skill_items: SkillItemAssets| {
				let inventory = Inventory::from([Some(item_handle.clone())]);

				assert_eq!(
					Some(SkillItem {
						token: Token::from("my item"),
						skill: Some(ItemSkill {
							token: Token::from("my skill"),
							icon: icon_handle.clone(),
							execution: SkillExecution::None
						})
					}),
					inventory.get_param_entry(&InventoryKey(0), &skill_items)
				);
			})
	}

	#[test]
	fn get_3rd_item() -> Result<(), RunSystemError> {
		let item_handle = new_handle();
		let skill_handle = new_handle();
		let icon_handle = new_handle();
		let mut app = setup(
			[(
				&item_handle,
				Item {
					token: Token::from("my item"),
					skill: Some(skill_handle.clone()),
					..default()
				},
			)],
			[(
				&skill_handle,
				Skill {
					token: Token::from("my skill"),
					icon: icon_handle.clone(),
					..default()
				},
			)],
		);

		app.world_mut()
			.run_system_once(move |skill_items: SkillItemAssets| {
				let inventory = Inventory::from([None, None, Some(item_handle.clone())]);

				assert_eq!(
					Some(SkillItem {
						token: Token::from("my item"),
						skill: Some(ItemSkill {
							token: Token::from("my skill"),
							icon: icon_handle.clone(),
							execution: SkillExecution::None
						})
					}),
					inventory.get_param_entry(&InventoryKey(2), &skill_items)
				);
			})
	}

	mod swap_internal {
		use super::*;

		#[test]
		fn swap() {
			let a = new_handle();
			let b = new_handle();
			let mut inventory = Inventory::from([Some(a.clone()), Some(b.clone())]);

			inventory.swap_internal(InventoryKey(0), InventoryKey(1));

			assert_eq!(Inventory::from([Some(b), Some(a)]), inventory)
		}

		#[test]
		fn swap_out_of_bounds() {
			let item = new_handle();
			let mut inventory = Inventory::from([Some(item.clone())]);

			inventory.swap_internal(InventoryKey(0), InventoryKey(1));

			assert_eq!(Inventory::from([None, Some(item)]), inventory)
		}

		#[test]
		fn swap_out_of_bounds_reverse() {
			let item = new_handle();
			let mut inventory = Inventory::from([Some(item.clone())]);

			inventory.swap_internal(InventoryKey(1), InventoryKey(0));

			assert_eq!(Inventory::from([None, Some(item)]), inventory)
		}
	}

	mod swap_external {
		use super::*;
		use common::tools::action_key::slot::{PlayerSlot, SlotKey};

		#[test]
		fn swap() {
			let a = new_handle();
			let b = new_handle();
			let mut inventory = Inventory::from([Some(a.clone())]);
			let mut slots = Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), Some(b.clone()))]);

			inventory.swap_external(&mut slots, InventoryKey(0), PlayerSlot::LOWER_R);

			assert_eq!(
				(
					Inventory::from([Some(b)]),
					Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), Some(a))])
				),
				(inventory, slots),
			)
		}

		#[test]
		fn swap_inventory_out_of_bounds() {
			let item = new_handle();
			let mut inventory = Inventory::from([]);
			let mut slots = Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), Some(item.clone()))]);

			inventory.swap_external(&mut slots, InventoryKey(4), PlayerSlot::LOWER_R);

			assert_eq!(
				(
					Inventory::from([None, None, None, None, Some(item)]),
					Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), None)])
				),
				(inventory, slots),
			)
		}

		#[test]
		fn swap_inventory_just_out_of_bounds() {
			let item = new_handle();
			let mut inventory = Inventory::from([None, None, None]);
			let mut slots = Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), Some(item.clone()))]);

			inventory.swap_external(&mut slots, InventoryKey(3), PlayerSlot::LOWER_R);

			assert_eq!(
				(
					Inventory::from([None, None, None, Some(item)]),
					Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), None)])
				),
				(inventory, slots),
			)
		}

		#[test]
		fn swap_slots_out_of_bounds() {
			let item = new_handle();
			let mut inventory = Inventory::from([Some(item.clone())]);
			let mut slots = Slots::from([]);

			inventory.swap_external(&mut slots, InventoryKey(0), PlayerSlot::LOWER_R);

			assert_eq!(
				(
					Inventory::from([None]),
					Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), Some(item))])
				),
				(inventory, slots),
			)
		}
	}
}
