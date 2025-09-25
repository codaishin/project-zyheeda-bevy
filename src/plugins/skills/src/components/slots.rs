pub(crate) mod visualization;

mod dto;

use crate::{
	components::{
		combos::Combos,
		inventory::Inventory,
		slots::{dto::SlotsDto, visualization::SlotVisualization},
	},
	item::{Item, ItemSkill, SkillItem},
	resources::{
		skill_item_assets::SkillItemAssets,
		skill_item_assets_usage::SkillItemAssetsUsage,
	},
	skills::{QueuedSkill, Skill},
	traits::peek_next::PeekNext,
};
use bevy::{asset::Handle, prelude::*};
use common::{
	tools::{
		action_key::slot::SlotKey,
		inventory_key::InventoryKey,
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::{GetFromSystemParam, GetRef},
		handles_loadout::{
			loadout::{LoadoutItem, LoadoutKey, SwapExternal, SwapInternal},
			slot_component::AvailableSkills,
		},
		iterate::Iterate,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
};
use macros::SavableComponent;
use std::{collections::HashMap, fmt::Debug};

#[derive(Component, SavableComponent, PartialEq, Debug, Clone)]
#[require(
	SlotVisualization<HandSlot>,
	SlotVisualization<ForearmSlot>,
	SlotVisualization<EssenceSlot>
)]
#[savable_component(dto = SlotsDto)]
pub struct Slots {
	pub(crate) self_entity: Option<Entity>,
	pub(crate) items: HashMap<SlotKey, Option<Handle<Item>>>,
}

impl Slots {
	#[cfg(test)]
	pub(crate) fn with_self_entity(mut self, entity: Entity) -> Self {
		self.self_entity = Some(entity);
		self
	}

	fn skill_item_with_execution(
		&self,
		key: &SlotKey,
		item: &Item,
		skill: Option<&Skill>,
		assets: &SkillItemAssetsUsage,
	) -> SkillItem {
		let Some((queue, combos)) = self.self_entity.and_then(|e| assets.usage.get(e).ok()) else {
			return Self::skill_item(item, skill, SkillExecution::None);
		};
		let mut queued_skills = queue.iterate().map(Self::match_combos(item, combos));

		if let Some((active, _)) = queued_skills.next().filter(|(_, k)| k == key) {
			return Self::skill_item(item, Some(active), SkillExecution::Active);
		}

		if let Some((queued, _)) = queued_skills.find(|(_, k)| k == key) {
			return Self::skill_item(item, Some(queued), SkillExecution::Queued);
		}

		let combo_skill = combos.peek_next(key, &item.item_type).or(skill);
		Self::skill_item(item, combo_skill, SkillExecution::None)
	}

	fn skill_item(item: &Item, skill: Option<&Skill>, execution: SkillExecution) -> SkillItem {
		SkillItem {
			token: item.token.clone(),
			skill: skill.map(|skill| ItemSkill {
				token: skill.token.clone(),
				icon: skill.icon.clone(),
				execution,
			}),
		}
	}

	fn match_combos<'a>(
		item: &'a Item,
		combos: &'a Combos,
	) -> impl Fn(&'a QueuedSkill) -> (&'a Skill, SlotKey) {
		|queued| match combos.peek_next(&queued.key, &item.item_type) {
			Some(combo_skill) => (combo_skill, queued.key),
			None => (&queued.skill, queued.key),
		}
	}
}

impl<T> From<T> for Slots
where
	T: IntoIterator<Item = (SlotKey, Option<Handle<Item>>)>,
{
	fn from(slots: T) -> Self {
		Self {
			self_entity: None,
			items: HashMap::from_iter(slots),
		}
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::from([])
	}
}

impl GetFromSystemParam<SlotKey> for Slots {
	type TParam<'w, 's> = SkillItemAssetsUsage<'w, 's>;
	type TItem<'i> = SkillItem;

	fn get_from_param(&self, key: &SlotKey, assets: &SkillItemAssetsUsage) -> Option<SkillItem> {
		let item = self
			.items
			.get(key)
			.and_then(|item| item.as_ref())
			.and_then(|item| assets.skill_item_assets.items.get(item))?;
		let skill = item
			.skill
			.as_ref()
			.and_then(|skill| assets.skill_item_assets.skills.get(skill));

		Some(self.skill_item_with_execution(key, item, skill, assets))
	}
}

impl GetFromSystemParam<AvailableSkills<SlotKey>> for Slots {
	type TParam<'w, 's> = SkillItemAssets<'w>;
	type TItem<'i> = Vec<Skill>;

	fn get_from_param(
		&self,
		AvailableSkills(key): &AvailableSkills<SlotKey>,
		SkillItemAssets { items, skills }: &SkillItemAssets,
	) -> Option<Vec<Skill>> {
		let item = self.items.get(key)?.as_ref()?;
		let item = items.get(item)?;

		let skills = skills
			.iter()
			.filter(|(_, skill)| skill.compatible_items.0.contains(&item.item_type))
			.map(|(_, skill)| skill.clone())
			.collect();

		Some(skills)
	}
}

impl LoadoutKey for Slots {
	type TKey = SlotKey;
}

impl LoadoutItem for Slots {
	type TItem = SkillItem;
}

impl SwapExternal<Inventory> for Slots {
	fn swap_external<TKey, TOtherKey>(&mut self, inventory: &mut Inventory, a: TKey, b: TOtherKey)
	where
		TKey: Into<SlotKey> + 'static,
		TOtherKey: Into<InventoryKey> + 'static,
	{
		inventory.swap_external(self, b, a);
	}
}

impl GetRef<SlotKey> for Slots {
	type TValue<'a>
		= &'a Handle<Item>
	where
		Self: 'a;

	fn get_ref(&self, key: &SlotKey) -> Option<&Handle<Item>> {
		let slot = self.items.get(key)?;
		slot.as_ref()
	}
}

impl SwapInternal for Slots {
	fn swap_internal<TKey>(&mut self, a: TKey, b: TKey)
	where
		TKey: Into<Self::TKey>,
	{
		let a: SlotKey = a.into();
		let b: SlotKey = b.into();
		if a == b {
			return;
		}
		let item_a = self.items.remove(&a).unwrap_or_default();
		let item_b = self.items.remove(&b).unwrap_or_default();
		self.items.insert(b, item_a);
		self.items.insert(a, item_b);
	}
}

impl<'a> Iterate<'a> for Slots {
	type TItem = (SlotKey, &'a Option<Handle<Item>>);
	type TIter = Iter<'a>;

	fn iterate(&'a self) -> Self::TIter {
		Iter {
			it: self.items.iter(),
		}
	}
}

pub struct Iter<'a> {
	it: std::collections::hash_map::Iter<'a, SlotKey, Option<Handle<Item>>>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = (SlotKey, &'a Option<Handle<Item>>);

	fn next(&mut self) -> Option<Self::Item> {
		let (key, slot) = self.it.next()?;
		Some((*key, slot))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skills::Skill;
	use common::tools::action_key::slot::PlayerSlot;
	use testing::{SingleThreadedApp, new_handle};

	mod get_handle {
		use super::*;

		#[test]
		fn get_some() {
			let item = new_handle();
			let slots = Slots::from([(SlotKey(2), Some(item.clone()))]);

			assert_eq!(Some(&item), slots.get_ref(&SlotKey(2)));
		}

		#[test]
		fn get_none() {
			let slots = Slots::from([(SlotKey(7), Some(new_handle()))]);

			assert_eq!(None::<&Handle<Item>>, slots.get_ref(&SlotKey(11)));
		}
	}

	mod get_skill_item {
		use super::*;
		use crate::{
			components::{combo_node::ComboNode, combos::Combos, queue::Queue},
			skills::QueuedSkill,
		};
		use bevy::ecs::system::{RunSystemError, RunSystemOnce};
		use common::{
			tools::{
				item_type::{CompatibleItems, ItemType},
				skill_execution::SkillExecution,
			},
			traits::handles_localization::Token,
		};

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
		fn get_none() -> Result<(), RunSystemError> {
			let mut app = setup([], []);

			app.world_mut()
				.run_system_once(|param: SkillItemAssetsUsage| {
					let slots = Slots::from([]);

					assert_eq!(
						None,
						slots.get_from_param(&SlotKey::from(PlayerSlot::UPPER_L), &param)
					);
				})
		}

		#[test]
		fn get_inactive() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				skill: Some(skill_handle.clone()),
				..default()
			};
			let skill = Skill {
				token: Token::from("my skill"),
				icon: icon_handle.clone(),
				..default()
			};
			let mut app = setup(
				[(&item_handle.clone(), item.clone())],
				[(&skill_handle.clone(), skill.clone())],
			);

			app.world_mut()
				.run_system_once(move |param: SkillItemAssetsUsage| {
					let slots = Slots::from([(
						SlotKey::from(PlayerSlot::UPPER_L),
						Some(item_handle.clone()),
					)]);

					assert_eq!(
						Some(SkillItem {
							token: Token::from("my item"),
							skill: Some(ItemSkill {
								token: Token::from("my skill"),
								icon: icon_handle.clone(),
								execution: SkillExecution::None
							})
						}),
						slots.get_from_param(&SlotKey::from(PlayerSlot::UPPER_L), &param)
					);
				})
		}

		#[test]
		fn get_active() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				skill: Some(skill_handle.clone()),
				..default()
			};
			let skill = Skill {
				token: Token::from("my skill"),
				icon: icon_handle.clone(),
				..default()
			};
			let mut app = setup(
				[(&item_handle.clone(), item.clone())],
				[(&skill_handle.clone(), skill.clone())],
			);
			let entity = app
				.world_mut()
				.spawn((
					Queue::from([QueuedSkill {
						key: SlotKey::from(PlayerSlot::UPPER_L),
						skill: skill.clone(),
						..default()
					}]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |param: SkillItemAssetsUsage| {
					let slots = Slots::from([(
						SlotKey::from(PlayerSlot::UPPER_L),
						Some(item_handle.clone()),
					)])
					.with_self_entity(entity);

					assert_eq!(
						Some(SkillItem {
							token: Token::from("my item"),
							skill: Some(ItemSkill {
								token: Token::from("my skill"),
								icon: icon_handle.clone(),
								execution: SkillExecution::Active
							})
						}),
						slots.get_from_param(&SlotKey::from(PlayerSlot::UPPER_L), &param)
					);
				})
		}

		#[test]
		fn get_inactive_when_first_queued_uses_other_slot_key() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				skill: Some(skill_handle.clone()),
				..default()
			};
			let skill = Skill {
				token: Token::from("my skill"),
				icon: icon_handle.clone(),
				..default()
			};
			let mut app = setup(
				[(&item_handle.clone(), item.clone())],
				[(&skill_handle.clone(), skill.clone())],
			);
			let entity = app
				.world_mut()
				.spawn((
					Queue::from([QueuedSkill {
						key: SlotKey::from(PlayerSlot::LOWER_L),
						..default()
					}]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |param: SkillItemAssetsUsage| {
					let slots = Slots::from([(
						SlotKey::from(PlayerSlot::UPPER_L),
						Some(item_handle.clone()),
					)])
					.with_self_entity(entity);

					assert_eq!(
						Some(SkillItem {
							token: Token::from("my item"),
							skill: Some(ItemSkill {
								token: Token::from("my skill"),
								icon: icon_handle.clone(),
								execution: SkillExecution::None
							})
						}),
						slots.get_from_param(&SlotKey::from(PlayerSlot::UPPER_L), &param)
					);
				})
		}

		#[test]
		fn get_queued() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				skill: Some(skill_handle.clone()),
				..default()
			};
			let skill = Skill {
				token: Token::from("my skill"),
				icon: icon_handle.clone(),
				..default()
			};
			let mut app = setup(
				[(&item_handle.clone(), item.clone())],
				[(&skill_handle.clone(), skill.clone())],
			);
			let entity = app
				.world_mut()
				.spawn((
					Queue::from([
						QueuedSkill {
							key: SlotKey::from(PlayerSlot::UPPER_R),
							..default()
						},
						QueuedSkill {
							key: SlotKey::from(PlayerSlot::UPPER_L),
							skill: skill.clone(),
							..default()
						},
					]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |param: SkillItemAssetsUsage| {
					let slots = Slots::from([(
						SlotKey::from(PlayerSlot::UPPER_L),
						Some(item_handle.clone()),
					)])
					.with_self_entity(entity);

					assert_eq!(
						Some(SkillItem {
							token: Token::from("my item"),
							skill: Some(ItemSkill {
								token: Token::from("my skill"),
								icon: icon_handle.clone(),
								execution: SkillExecution::Queued
							})
						}),
						slots.get_from_param(&SlotKey::from(PlayerSlot::UPPER_L), &param)
					);
				})
		}

		#[test]
		fn get_inactive_from_combo() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				skill: Some(skill_handle.clone()),
				item_type: ItemType::ForceEssence,
				..default()
			};
			let skill = Skill {
				token: Token::from("my skill"),
				compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
				icon: icon_handle.clone(),
				..default()
			};
			let combo_skill = Skill {
				token: Token::from("my combo skill"),
				compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
				icon: icon_handle.clone(),
				..default()
			};
			let mut app = setup(
				[(&item_handle.clone(), item.clone())],
				[(&skill_handle.clone(), skill.clone())],
			);
			let entity = app
				.world_mut()
				.spawn((
					Queue::from([]),
					Combos::from(ComboNode::new([(
						SlotKey::from(PlayerSlot::UPPER_L),
						(combo_skill.clone(), ComboNode::default()),
					)])),
				))
				.id();

			app.world_mut()
				.run_system_once(move |param: SkillItemAssetsUsage| {
					let slots = Slots::from([(
						SlotKey::from(PlayerSlot::UPPER_L),
						Some(item_handle.clone()),
					)])
					.with_self_entity(entity);

					assert_eq!(
						Some(SkillItem {
							token: Token::from("my item"),
							skill: Some(ItemSkill {
								token: Token::from("my combo skill"),
								icon: icon_handle.clone(),
								execution: SkillExecution::None
							})
						}),
						slots.get_from_param(&SlotKey::from(PlayerSlot::UPPER_L), &param)
					);
				})
		}

		#[test]
		fn get_active_from_combo() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				skill: Some(skill_handle.clone()),
				item_type: ItemType::ForceEssence,
				..default()
			};
			let skill = Skill {
				token: Token::from("my skill"),
				compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
				icon: icon_handle.clone(),
				..default()
			};
			let combo_skill = Skill {
				token: Token::from("my combo skill"),
				compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
				icon: icon_handle.clone(),
				..default()
			};
			let mut app = setup(
				[(&item_handle.clone(), item.clone())],
				[(&skill_handle.clone(), skill.clone())],
			);
			let entity = app
				.world_mut()
				.spawn((
					Queue::from([QueuedSkill {
						key: SlotKey::from(PlayerSlot::UPPER_L),
						skill: skill.clone(),
						..default()
					}]),
					Combos::from(ComboNode::new([(
						SlotKey::from(PlayerSlot::UPPER_L),
						(combo_skill.clone(), ComboNode::default()),
					)])),
				))
				.id();

			app.world_mut()
				.run_system_once(move |param: SkillItemAssetsUsage| {
					let slots = Slots::from([(
						SlotKey::from(PlayerSlot::UPPER_L),
						Some(item_handle.clone()),
					)])
					.with_self_entity(entity);

					assert_eq!(
						Some(SkillItem {
							token: Token::from("my item"),
							skill: Some(ItemSkill {
								token: Token::from("my combo skill"),
								icon: icon_handle.clone(),
								execution: SkillExecution::Active
							})
						}),
						slots.get_from_param(&SlotKey::from(PlayerSlot::UPPER_L), &param)
					);
				})
		}

		#[test]
		fn get_queued_from_combo() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let item = Item {
				token: Token::from("my item"),
				skill: Some(skill_handle.clone()),
				item_type: ItemType::ForceEssence,
				..default()
			};
			let skill = Skill {
				token: Token::from("my skill"),
				compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
				icon: icon_handle.clone(),
				..default()
			};
			let combo_skill = Skill {
				token: Token::from("my combo skill"),
				compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
				icon: icon_handle.clone(),
				..default()
			};
			let mut app = setup(
				[(&item_handle.clone(), item.clone())],
				[(&skill_handle.clone(), skill.clone())],
			);
			let entity = app
				.world_mut()
				.spawn((
					Queue::from([
						QueuedSkill {
							key: SlotKey::from(PlayerSlot::LOWER_L),
							..default()
						},
						QueuedSkill {
							key: SlotKey::from(PlayerSlot::UPPER_L),
							skill: skill.clone(),
							..default()
						},
					]),
					Combos::from(ComboNode::new([(
						SlotKey::from(PlayerSlot::UPPER_L),
						(combo_skill.clone(), ComboNode::default()),
					)])),
				))
				.id();

			app.world_mut()
				.run_system_once(move |param: SkillItemAssetsUsage| {
					let slots = Slots::from([(
						SlotKey::from(PlayerSlot::UPPER_L),
						Some(item_handle.clone()),
					)])
					.with_self_entity(entity);

					assert_eq!(
						Some(SkillItem {
							token: Token::from("my item"),
							skill: Some(ItemSkill {
								token: Token::from("my combo skill"),
								icon: icon_handle.clone(),
								execution: SkillExecution::Queued
							})
						}),
						slots.get_from_param(&SlotKey::from(PlayerSlot::UPPER_L), &param)
					);
				})
		}
	}

	mod swap_internal {
		use super::*;

		#[test]
		fn swap() {
			let a = new_handle();
			let b = new_handle();
			let mut slots = Slots::from([
				(SlotKey::from(PlayerSlot::LOWER_R), Some(a.clone())),
				(SlotKey::from(PlayerSlot::UPPER_R), Some(b.clone())),
			]);

			slots.swap_internal(PlayerSlot::LOWER_R, PlayerSlot::UPPER_R);

			assert_eq!(
				Slots::from([
					(SlotKey::from(PlayerSlot::LOWER_R), Some(b)),
					(SlotKey::from(PlayerSlot::UPPER_R), Some(a)),
				]),
				slots,
			);
		}

		#[test]
		fn swap_when_one_slot_missing() {
			let item = new_handle();
			let mut slots = Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), Some(item.clone()))]);

			slots.swap_internal(PlayerSlot::LOWER_R, PlayerSlot::UPPER_R);

			assert_eq!(
				Slots::from([
					(SlotKey::from(PlayerSlot::LOWER_R), None),
					(SlotKey::from(PlayerSlot::UPPER_R), Some(item)),
				]),
				slots,
			);
		}
	}

	#[test]
	fn swap_same_key() {
		let a = new_handle();
		let mut slots = Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), Some(a.clone()))]);

		slots.swap_internal(PlayerSlot::LOWER_R, PlayerSlot::LOWER_R);

		assert_eq!(
			Slots::from([(SlotKey::from(PlayerSlot::LOWER_R), Some(a))]),
			slots,
		);
	}

	mod available_skills {
		use super::*;
		use bevy::ecs::system::{RunSystemError, RunSystemOnce};
		use common::{
			tools::item_type::{CompatibleItems, ItemType},
			traits::handles_localization::Token,
		};
		use testing::assert_eq_unordered;

		fn setup<const I: usize, const S: usize>(
			items: [(Handle<Item>, Item); I],
			skills: [(Handle<Skill>, Skill); S],
		) -> App {
			let mut app = App::new().single_threaded(Update);
			let mut item_assets = Assets::default();
			let mut skill_assets = Assets::default();

			for (handle, asset) in items {
				item_assets.insert(&handle, asset);
			}

			for (handle, asset) in skills {
				skill_assets.insert(&handle, asset);
			}

			app.insert_resource(item_assets);
			app.insert_resource(skill_assets);

			app
		}

		#[test]
		fn get_skills_compatible_with_slot_item() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let item = Item {
				item_type: ItemType::ForceEssence,
				..default()
			};
			let skills = [
				(
					new_handle(),
					Skill {
						token: Token::from("essence"),
						compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
						..default()
					},
				),
				(
					new_handle(),
					Skill {
						token: Token::from("pistol"),
						compatible_items: CompatibleItems::from([ItemType::Pistol]),
						..default()
					},
				),
				(
					new_handle(),
					Skill {
						token: Token::from("essence and pistol"),
						compatible_items: CompatibleItems::from([
							ItemType::ForceEssence,
							ItemType::Pistol,
						]),
						..default()
					},
				),
			];
			let mut app = setup([(item_handle.clone(), item)], skills.clone());

			app.world_mut()
				.run_system_once(move |param: SkillItemAssets| {
					let slots = Slots::from([(
						SlotKey::from(PlayerSlot::UPPER_L),
						Some(item_handle.clone()),
					)]);

					let available = slots.get_from_param(
						&AvailableSkills(SlotKey::from(PlayerSlot::UPPER_L)),
						&param,
					);

					assert_eq_unordered!(
						Some(vec![
							Skill {
								token: Token::from("essence"),
								compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
								..default()
							},
							Skill {
								token: Token::from("essence and pistol"),
								compatible_items: CompatibleItems::from([
									ItemType::ForceEssence,
									ItemType::Pistol,
								]),
								..default()
							}
						]),
						available
					);
				})
		}
	}
}
