use crate::systems::slot::visualization::track_slots::GetSlotDefinition;
use bevy::prelude::*;
use common::prelude::*;
use std::{collections::HashMap, ops::Deref};

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct SlotDefinitions {
	pub(crate) forearms: HashMap<BoneName, SlotKey>,
	pub(crate) hands: HashMap<BoneName, SlotKey>,
	pub(crate) essences: HashMap<MeshName, SlotKey>,
}

impl GetSlotDefinition<VisibleForearmSlot> for SlotDefinitions {
	fn get_slot_definition(&self, name: &str) -> Option<VisibleForearmSlot> {
		self.forearms.get(name).copied().map(VisibleForearmSlot)
	}
}

impl GetSlotDefinition<VisibleHandSlot> for SlotDefinitions {
	fn get_slot_definition(&self, name: &str) -> Option<VisibleHandSlot> {
		self.hands.get(name).copied().map(VisibleHandSlot)
	}
}

impl GetSlotDefinition<VisibleEssenceSlot> for SlotDefinitions {
	fn get_slot_definition(&self, name: &str) -> Option<VisibleEssenceSlot> {
		let mut best_match: Option<(&MeshName, &SlotKey)> = None;

		for (mesh_name, slot_key) in &self.essences {
			if !name.starts_with(mesh_name.deref()) {
				continue;
			}

			if matches!(best_match, Some((best_name, ..)) if best_name.len() > mesh_name.len()) {
				continue;
			}

			best_match = Some((mesh_name, slot_key));
		}

		best_match.map(|(.., slot_key)| VisibleEssenceSlot(*slot_key))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	mod forearm {
		use super::*;

		#[test]
		fn get_slot() {
			let slots = SlotDefinitions {
				forearms: HashMap::from([(BoneName::from("bone"), SlotKey(11))]),
				..default()
			};

			let slot = GetSlotDefinition::<VisibleForearmSlot>::get_slot_definition(&slots, "bone");

			assert_eq!(Some(VisibleForearmSlot(SlotKey(11))), slot);
		}

		#[test]
		fn get_none() {
			let slots = SlotDefinitions {
				forearms: HashMap::from([(BoneName::from("bone"), SlotKey(11))]),
				..default()
			};

			let slot =
				GetSlotDefinition::<VisibleForearmSlot>::get_slot_definition(&slots, "other bone");

			assert_eq!(None, slot);
		}
	}

	mod hand {
		use super::*;

		#[test]
		fn get_slot() {
			let slots = SlotDefinitions {
				hands: HashMap::from([(BoneName::from("bone"), SlotKey(11))]),
				..default()
			};

			let slot = GetSlotDefinition::<VisibleHandSlot>::get_slot_definition(&slots, "bone");

			assert_eq!(Some(VisibleHandSlot(SlotKey(11))), slot);
		}

		#[test]
		fn get_none() {
			let slots = SlotDefinitions {
				forearms: HashMap::from([(BoneName::from("bone"), SlotKey(11))]),
				..default()
			};

			let slot =
				GetSlotDefinition::<VisibleHandSlot>::get_slot_definition(&slots, "other bone");

			assert_eq!(None, slot);
		}
	}

	mod essence {
		use testing::repeat_scope;

		use super::*;

		#[test]
		fn get_slot() {
			let slots = SlotDefinitions {
				essences: HashMap::from([(MeshName::from("bone"), SlotKey(11))]),
				..default()
			};

			let slot = GetSlotDefinition::<VisibleEssenceSlot>::get_slot_definition(&slots, "bone");

			assert_eq!(Some(VisibleEssenceSlot(SlotKey(11))), slot);
		}

		#[test]
		fn get_none() {
			let slots = SlotDefinitions {
				forearms: HashMap::from([(BoneName::from("bone"), SlotKey(11))]),
				..default()
			};

			let slot =
				GetSlotDefinition::<VisibleEssenceSlot>::get_slot_definition(&slots, "other bone");

			assert_eq!(None, slot);
		}

		#[test]
		fn get_slot_with_material_remainder() {
			let slots = SlotDefinitions {
				essences: HashMap::from([(MeshName::from("bone"), SlotKey(11))]),
				..default()
			};

			let slot = GetSlotDefinition::<VisibleEssenceSlot>::get_slot_definition(
				&slots,
				"bone.some_material",
			);

			assert_eq!(Some(VisibleEssenceSlot(SlotKey(11))), slot);
		}

		#[test]
		fn get_slot_with_material_remainder_and_dot_on_name() {
			let slots = SlotDefinitions {
				essences: HashMap::from([(MeshName::from("bone.L"), SlotKey(11))]),
				..default()
			};

			let slot = GetSlotDefinition::<VisibleEssenceSlot>::get_slot_definition(
				&slots,
				"bone.L.some_material",
			);

			assert_eq!(Some(VisibleEssenceSlot(SlotKey(11))), slot);
		}

		#[test]
		fn get_best_match() {
			repeat_scope!(10, {
				let slots = SlotDefinitions {
					essences: HashMap::from([
						(MeshName::from("bone"), SlotKey(11)),
						(MeshName::from("bone.L"), SlotKey(12)),
					]),
					..default()
				};

				let slot = GetSlotDefinition::<VisibleEssenceSlot>::get_slot_definition(
					&slots,
					"bone.L.some_material",
				);

				assert_eq!(Some(VisibleEssenceSlot(SlotKey(12))), slot);
			});
		}
	}
}
