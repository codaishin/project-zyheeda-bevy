use crate::{
	components::{combos::Combos, inventory::Inventory, queue::Queue, slots::Slots},
	item::Item,
	skills::Skill,
	system_parameters::loadout::LoadoutReader,
	traits::peek_next::PeekNext,
};
use bevy::prelude::*;
use common::{
	tools::{inventory_key::InventoryKey, skill_execution::SkillExecution},
	traits::{
		accessors::get::{ContextChanged, EntityContext, GetProperty},
		handles_loadout::{
			LoadoutKey,
			skills::{ReadSkills, SkillIcon, SkillToken, Skills},
		},
		handles_localization::Token,
		iterate::Iterate,
	},
};

impl EntityContext<Skills> for LoadoutReader<'_, '_> {
	type TContext<'ctx> = SkillsView<'ctx>;

	fn get_entity_context<'ctx>(
		param: &'ctx LoadoutReader,
		entity: Entity,
		_: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let (slots, inventory, combos, queue) = param.agents.get(entity).ok()?;

		Some(SkillsView {
			inventory,
			slots,
			queue,
			combos,
			items: &param.items,
			skills: &param.skills,
		})
	}
}

pub struct SkillsView<'ctx> {
	inventory: Ref<'ctx, Inventory>,
	slots: Ref<'ctx, Slots>,
	queue: Ref<'ctx, Queue>,
	combos: Ref<'ctx, Combos>,
	items: &'ctx Assets<Item>,
	skills: &'ctx Assets<Skill>,
}

impl ContextChanged for SkillsView<'_> {
	fn context_changed(&self) -> bool {
		self.inventory.is_changed()
			|| self.slots.is_changed()
			|| self.queue.is_changed()
			|| self.combos.is_changed()
	}
}

impl ReadSkills for SkillsView<'_> {
	type TSkill<'a>
		= ReadSkill
	where
		Self: 'a;

	fn get_skill<TKey>(&self, key: TKey) -> Option<Self::TSkill<'_>>
	where
		TKey: Into<LoadoutKey>,
	{
		let key = key.into();
		let handle = match key {
			LoadoutKey::Inventory(InventoryKey(i)) => self.inventory.0.get(i)?.as_ref()?,
			LoadoutKey::Slot(slot) => self.slots.items.get(&slot)?.as_ref()?,
		};
		let item = self.items.get(handle)?;
		let skill_handle = item.skill.as_ref()?;
		let mut skill = self.skills.get(skill_handle)?;

		let LoadoutKey::Slot(slot_key) = key else {
			return Some(ReadSkill {
				token: skill.token.clone(),
				icon: skill.icon.clone(),
				execution: SkillExecution::None,
			});
		};

		let mut queue = self.queue.iterate();

		if let Some(active) = queue.next().filter(|q| slot_key == q.key) {
			return Some(ReadSkill {
				token: active.skill.token.clone(),
				icon: active.skill.icon.clone(),
				execution: SkillExecution::Active,
			});
		}

		if let Some(queued) = queue.find(|q| slot_key == q.key) {
			return Some(ReadSkill {
				token: queued.skill.token.clone(),
				icon: queued.skill.icon.clone(),
				execution: SkillExecution::Queued,
			});
		}

		if let Some(combo_skill) = self.combos.peek_next(&slot_key, &item.item_type) {
			skill = combo_skill;
		}

		Some(ReadSkill {
			token: skill.token.clone(),
			icon: skill.icon.clone(),
			execution: SkillExecution::None,
		})
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReadSkill {
	token: Token,
	icon: Handle<Image>,
	execution: SkillExecution,
}

impl GetProperty<SkillToken> for ReadSkill {
	fn get_property(&self) -> &'_ Token {
		&self.token
	}
}

impl GetProperty<SkillIcon> for ReadSkill {
	fn get_property(&self) -> &'_ Handle<Image> {
		&self.icon
	}
}

impl GetProperty<SkillExecution> for ReadSkill {
	fn get_property(&self) -> SkillExecution {
		self.execution
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::combos::Combos, skills::Skill};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::{action_key::slot::SlotKey, inventory_key::InventoryKey};
	use testing::{SingleThreadedApp, new_handle};

	mod get_skill {
		use super::*;
		use crate::{
			components::combo_node::ComboNode,
			skills::{QueuedSkill, SkillMode},
		};
		use common::tools::item_type::{CompatibleItems, ItemType};

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
		fn inventory_skill() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let skill = Skill {
				token: Token::from("my skill"),
				icon: icon_handle.clone(),
				..default()
			};
			let item = Item {
				skill: Some(skill_handle.clone()),
				..default()
			};
			let mut app = setup([(&item_handle, item)], [(&skill_handle, skill)]);
			let entity = app
				.world_mut()
				.spawn((
					Inventory::from([None, None, None, Some(item_handle), None]),
					Slots::default(),
					Combos::default(),
					Queue::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx = LoadoutReader::get_entity_context(&loadout, entity, Skills).unwrap();
					let item = ctx.get_skill(InventoryKey(3));

					assert_eq!(
						Some(ReadSkill {
							token: Token::from("my skill"),
							icon: icon_handle.clone(),
							execution: SkillExecution::None,
						}),
						item
					);
				})
		}

		#[test]
		fn slot_skill() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let skill = Skill {
				token: Token::from("my skill"),
				icon: icon_handle.clone(),
				..default()
			};
			let item = Item {
				skill: Some(skill_handle.clone()),
				..default()
			};
			let mut app = setup([(&item_handle, item)], [(&skill_handle, skill)]);
			let entity = app
				.world_mut()
				.spawn((
					Inventory::default(),
					Slots::from([(SlotKey(11), Some(item_handle))]),
					Combos::default(),
					Queue::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx = LoadoutReader::get_entity_context(&loadout, entity, Skills).unwrap();
					let item = ctx.get_skill(SlotKey(11));

					assert_eq!(
						Some(ReadSkill {
							token: Token::from("my skill"),
							icon: icon_handle.clone(),
							execution: SkillExecution::None,
						}),
						item
					);
				})
		}

		#[test]
		fn queued_skill() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let skill = Skill {
				token: Token::from("my skill"),
				icon: new_handle(),
				..default()
			};
			let item = Item {
				skill: Some(skill_handle.clone()),
				..default()
			};
			let mut app = setup([(&item_handle, item)], [(&skill_handle, skill)]);
			let entity = app
				.world_mut()
				.spawn((
					Inventory::default(),
					Slots::from([(SlotKey(11), Some(item_handle))]),
					Combos::default(),
					Queue::from([
						QueuedSkill {
							key: SlotKey(42),
							skill: Skill { ..default() },
							skill_mode: SkillMode::Hold,
						},
						QueuedSkill {
							key: SlotKey(11),
							skill: Skill {
								token: Token::from("my queued skill"),
								icon: icon_handle.clone(),
								..default()
							},
							skill_mode: SkillMode::Hold,
						},
					]),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx = LoadoutReader::get_entity_context(&loadout, entity, Skills).unwrap();
					let item = ctx.get_skill(SlotKey(11));

					assert_eq!(
						Some(ReadSkill {
							token: Token::from("my queued skill"),
							icon: icon_handle.clone(),
							execution: SkillExecution::Queued,
						}),
						item
					);
				})
		}

		#[test]
		fn active_skill() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let skill = Skill {
				token: Token::from("my skill"),
				icon: new_handle(),
				..default()
			};
			let item = Item {
				skill: Some(skill_handle.clone()),
				..default()
			};
			let mut app = setup([(&item_handle, item)], [(&skill_handle, skill)]);
			let entity = app
				.world_mut()
				.spawn((
					Inventory::default(),
					Slots::from([(SlotKey(11), Some(item_handle))]),
					Combos::default(),
					Queue::from([QueuedSkill {
						key: SlotKey(11),
						skill: Skill {
							token: Token::from("my active skill"),
							icon: icon_handle.clone(),
							..default()
						},
						skill_mode: SkillMode::Hold,
					}]),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx = LoadoutReader::get_entity_context(&loadout, entity, Skills).unwrap();
					let item = ctx.get_skill(SlotKey(11));

					assert_eq!(
						Some(ReadSkill {
							token: Token::from("my active skill"),
							icon: icon_handle.clone(),
							execution: SkillExecution::Active,
						}),
						item
					);
				})
		}

		#[test]
		fn active_skill_when_also_same_slot_queued() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let skill = Skill {
				token: Token::from("my skill"),
				icon: new_handle(),
				..default()
			};
			let item = Item {
				skill: Some(skill_handle.clone()),
				..default()
			};
			let mut app = setup([(&item_handle, item)], [(&skill_handle, skill)]);
			let entity = app
				.world_mut()
				.spawn((
					Inventory::default(),
					Slots::from([(SlotKey(11), Some(item_handle))]),
					Combos::default(),
					Queue::from([
						QueuedSkill {
							key: SlotKey(11),
							skill: Skill {
								token: Token::from("my active skill"),
								icon: icon_handle.clone(),
								..default()
							},
							skill_mode: SkillMode::Hold,
						},
						QueuedSkill {
							key: SlotKey(11),
							skill: Skill {
								token: Token::from("my queued skill"),
								icon: icon_handle.clone(),
								..default()
							},
							skill_mode: SkillMode::Hold,
						},
					]),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx = LoadoutReader::get_entity_context(&loadout, entity, Skills).unwrap();
					let item = ctx.get_skill(SlotKey(11));

					assert_eq!(
						Some(ReadSkill {
							token: Token::from("my active skill"),
							icon: icon_handle.clone(),
							execution: SkillExecution::Active,
						}),
						item
					);
				})
		}

		#[test]
		fn combo_skill() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let skill_handle = new_handle();
			let icon_handle = new_handle();
			let skill = Skill {
				token: Token::from("my skill"),
				icon: new_handle(),
				compatible_items: CompatibleItems::from([ItemType::VoidBeam]),
				..default()
			};
			let item = Item {
				skill: Some(skill_handle.clone()),
				item_type: ItemType::VoidBeam,
				..default()
			};
			let mut app = setup([(&item_handle, item)], [(&skill_handle, skill)]);
			let entity = app
				.world_mut()
				.spawn((
					Inventory::default(),
					Slots::from([(SlotKey(11), Some(item_handle))]),
					Combos::from(ComboNode::new([(
						SlotKey(11),
						(
							Skill {
								token: Token::from("my combo skill"),
								icon: icon_handle.clone(),
								compatible_items: CompatibleItems::from([ItemType::VoidBeam]),
								..default()
							},
							ComboNode::default(),
						),
					)])),
					Queue::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx = LoadoutReader::get_entity_context(&loadout, entity, Skills).unwrap();
					let item = ctx.get_skill(SlotKey(11));

					assert_eq!(
						Some(ReadSkill {
							token: Token::from("my combo skill"),
							icon: icon_handle.clone(),
							execution: SkillExecution::None,
						}),
						item
					);
				})
		}
	}

	mod skill {
		use super::*;
		use common::traits::accessors::get::DynProperty;

		#[test]
		fn get_token() {
			let skill = ReadSkill {
				token: Token::from("my skill"),
				icon: new_handle(),
				execution: SkillExecution::None,
			};

			assert_eq!(&Token::from("my skill"), skill.dyn_property::<SkillToken>());
		}

		#[test]
		fn get_icon() {
			let skill = ReadSkill {
				token: Token::from("my skill"),
				icon: new_handle(),
				execution: SkillExecution::None,
			};

			assert_eq!(&skill.icon, skill.dyn_property::<SkillIcon>());
		}

		#[test]
		fn get_execution() {
			let skill = ReadSkill {
				token: Token::from("my skill"),
				icon: new_handle(),
				execution: SkillExecution::Queued,
			};

			assert_eq!(
				SkillExecution::Queued,
				skill.dyn_property::<SkillExecution>(),
			);
		}
	}
}
