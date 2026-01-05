use crate::{
	components::slots::Slots,
	item::Item,
	skills::{Skill, SkillId},
	system_parameters::loadout::LoadoutReader,
};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{ContextChanged, GetContext, GetProperty},
		handles_loadout::{
			available_skills::{AvailableSkills, ReadAvailableSkills},
			skills::{GetSkillId, SkillIcon, SkillToken},
		},
		handles_localization::Token,
	},
};

impl GetContext<AvailableSkills> for LoadoutReader<'_, '_> {
	type TContext<'ctx> = AvailableSkillsView<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx LoadoutReader,
		AvailableSkills { entity }: AvailableSkills,
	) -> Option<Self::TContext<'ctx>> {
		let (slots, ..) = param.agents.get(entity).ok()?;

		Some(AvailableSkillsView {
			slots,
			items: &param.items,
			skills: &param.skills,
		})
	}
}

pub struct AvailableSkillsView<'ctx> {
	slots: Ref<'ctx, Slots>,
	items: &'ctx Assets<Item>,
	skills: &'ctx Assets<Skill>,
}

impl ReadAvailableSkills<SkillId> for AvailableSkillsView<'_> {
	type TSkill<'a>
		= ReadAvailableSkill
	where
		Self: 'a;

	fn get_available_skills(&self, key: SlotKey) -> impl Iterator<Item = Self::TSkill<'_>> {
		let Some(Some(item_handle)) = self.slots.items.get(&key) else {
			return AvailableSkillsIterator::Empty;
		};
		let Some(item) = self.items.get(item_handle) else {
			return AvailableSkillsIterator::Empty;
		};

		AvailableSkillsIterator::Compatible {
			skills: self.skills.iter(),
			item,
		}
	}
}

impl ContextChanged for AvailableSkillsView<'_> {
	fn context_changed(&self) -> bool {
		self.slots.is_changed()
	}
}

enum AvailableSkillsIterator<'a, TSkills> {
	Empty,
	Compatible { skills: TSkills, item: &'a Item },
}

impl<'a, TSkills> Iterator for AvailableSkillsIterator<'a, TSkills>
where
	TSkills: Iterator<Item = (AssetId<Skill>, &'a Skill)>,
{
	type Item = ReadAvailableSkill;

	fn next(&mut self) -> Option<Self::Item> {
		let Self::Compatible { skills, item } = self else {
			return None;
		};

		loop {
			let (_, skill) = skills.next()?;

			if !skill.compatible_items.0.contains(&item.item_type) {
				continue;
			}

			return Some(ReadAvailableSkill {
				id: skill.id,
				token: skill.token.clone(),
				icon: skill.icon.clone(),
			});
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct ReadAvailableSkill {
	id: SkillId,
	token: Token,
	icon: Handle<Image>,
}

impl GetProperty<SkillToken> for ReadAvailableSkill {
	fn get_property(&self) -> &'_ Token {
		&self.token
	}
}

impl GetProperty<SkillIcon> for ReadAvailableSkill {
	fn get_property(&self) -> &'_ Handle<Image> {
		&self.icon
	}
}

impl GetSkillId<SkillId> for ReadAvailableSkill {
	fn get_skill_id(&self) -> SkillId {
		self.id
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::combos::Combos, skills::Skill};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::action_key::slot::SlotKey;
	use testing::{SingleThreadedApp, new_handle};

	mod get_skills {
		use super::*;
		use crate::components::{inventory::Inventory, queue::Queue};
		use common::tools::item_type::{CompatibleItems, ItemType};
		use testing::assert_eq_unordered;
		use uuid::Uuid;

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
		fn matching_with_item() -> Result<(), RunSystemError> {
			let item_handle = new_handle();
			let id_a = SkillId(Uuid::new_v4());
			let id_b = SkillId(Uuid::new_v4());
			let icon_handle_a = new_handle();
			let icon_handle_b = new_handle();
			let compatible_a = Skill {
				id: id_a,
				token: Token::from("my compatible skill a"),
				icon: icon_handle_a.clone(),
				compatible_items: CompatibleItems::from([ItemType::Pistol, ItemType::VoidBeam]),
				..default()
			};
			let compatible_b = Skill {
				id: id_b,
				token: Token::from("my compatible skill b"),
				icon: icon_handle_b.clone(),
				compatible_items: CompatibleItems::from([ItemType::ForceEssence, ItemType::Pistol]),
				..default()
			};
			let incompatible = Skill {
				token: Token::from("my incompatible skill"),
				icon: new_handle(),
				compatible_items: CompatibleItems::from([ItemType::VoidBeam]),
				..default()
			};
			let item = Item {
				item_type: ItemType::Pistol,
				..default()
			};
			let mut app = setup(
				[(&item_handle, item)],
				[
					(&new_handle(), compatible_a),
					(&new_handle(), compatible_b),
					(&new_handle(), incompatible),
				],
			);
			let entity = app
				.world_mut()
				.spawn((
					Inventory::default(),
					Slots::from([(SlotKey(44), Some(item_handle))]),
					Combos::default(),
					Queue::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |loadout: LoadoutReader| {
					let ctx =
						LoadoutReader::get_context(&loadout, AvailableSkills { entity }).unwrap();
					let skills = ctx.get_available_skills(SlotKey(44));

					assert_eq_unordered!(
						vec![
							ReadAvailableSkill {
								id: id_a,
								token: Token::from("my compatible skill a"),
								icon: icon_handle_a.clone(),
							},
							ReadAvailableSkill {
								id: id_b,
								token: Token::from("my compatible skill b"),
								icon: icon_handle_b.clone(),
							}
						],
						skills.collect::<Vec<_>>(),
					);
				})
		}
	}

	mod skill {
		use super::*;
		use common::traits::accessors::get::DynProperty;
		use uuid::Uuid;

		#[test]
		fn get_token() {
			let skill = ReadAvailableSkill {
				id: SkillId(Uuid::new_v4()),
				token: Token::from("my skill"),
				icon: new_handle(),
			};

			assert_eq!(&Token::from("my skill"), skill.dyn_property::<SkillToken>());
		}

		#[test]
		fn get_icon() {
			let skill = ReadAvailableSkill {
				id: SkillId(Uuid::new_v4()),
				token: Token::from("my skill"),
				icon: new_handle(),
			};

			assert_eq!(&skill.icon, skill.dyn_property::<SkillIcon>());
		}

		#[test]
		fn get_id() {
			let skill = ReadAvailableSkill {
				id: SkillId(Uuid::new_v4()),
				token: Token::from("my skill"),
				icon: new_handle(),
			};

			assert_eq!(skill.id, skill.get_skill_id());
		}
	}
}
