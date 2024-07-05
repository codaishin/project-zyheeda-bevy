use super::{Item, Slot};
use crate::{
	items::{slot_key::SlotKey, SkillHandle},
	skills::Skill,
};
use bevy::{asset::Handle, ecs::component::Component};
use common::traits::get::Get;
use std::collections::HashMap;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Slots(pub HashMap<SlotKey, Slot>);

impl Slots {
	pub fn new() -> Self {
		Self(HashMap::new())
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::new()
	}
}

impl Get<SlotKey, Item> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Item> {
		let slot = self.0.get(key)?;
		slot.item.as_ref()
	}
}

impl Get<SlotKey, Handle<Skill>> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Handle<Skill>> {
		let item: &Item = self.get(key)?;

		match &item.skill {
			SkillHandle::Handle(handle) => Some(handle),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Mounts;
	use bevy::{
		asset::AssetId,
		prelude::Entity,
		utils::{default, Uuid},
	};
	use common::{components::Side, traits::load_asset::Path};

	fn mounts() -> Mounts<Entity> {
		Mounts {
			hand: Entity::from_raw(100),
			forearm: Entity::from_raw(200),
		}
	}

	#[test]
	fn get_off_hand() {
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: mounts(),
					item: Some(Item {
						name: "my item",
						..default()
					}),
				},
			)]
			.into(),
		);

		assert_eq!(
			Some(&Item {
				name: "my item",
				..default()
			}),
			slots.get(&SlotKey::Hand(Side::Off))
		);
	}

	#[test]
	fn get_main_hand() {
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: mounts(),
					item: Some(Item {
						name: "my item",
						..default()
					}),
				},
			)]
			.into(),
		);

		assert_eq!(
			Some(&Item {
				name: "my item",
				..default()
			}),
			slots.get(&SlotKey::Hand(Side::Main))
		);
	}

	#[test]
	fn get_none() {
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: mounts(),
					item: Some(Item {
						name: "my item",
						..default()
					}),
				},
			)]
			.into(),
		);

		assert_eq!(None::<&Item>, slots.get(&SlotKey::Hand(Side::Off)));
	}

	#[test]
	fn get_skill_handle() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: mounts(),
					item: Some(Item {
						name: "my item",
						skill: SkillHandle::Handle(handle.clone()),
						..default()
					}),
				},
			)]
			.into(),
		);

		assert_eq!(Some(&handle), slots.get(&SlotKey::Hand(Side::Main)));
	}

	#[test]
	fn get_skill_handle_none_when_not_skill_handle_stored() {
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: mounts(),
					item: Some(Item {
						name: "my item",
						skill: SkillHandle::Path(Path::from("some/skill/path")),
						..default()
					}),
				},
			)]
			.into(),
		);

		assert_eq!(
			None::<&Handle<Skill>>,
			slots.get(&SlotKey::Hand(Side::Main))
		);
	}
}
