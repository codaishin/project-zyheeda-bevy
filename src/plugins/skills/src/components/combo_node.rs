use bevy::{ecs::component::Component, prelude::default};
use common::traits::{
	get::{Get, GetMut},
	insert::TryInsert,
	iterate::Iterate,
};
use std::collections::{hash_map::Entry, HashMap};

use crate::{items::slot_key::SlotKey, skills::Skill, traits::PeekNext};

use super::slots::Slots;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct ComboNode<TSkill = Skill>(HashMap<SlotKey, (TSkill, ComboNode<TSkill>)>);

impl<TSkill> ComboNode<TSkill> {
	pub fn new<const N: usize>(combos: [(SlotKey, (TSkill, ComboNode<TSkill>)); N]) -> Self {
		Self(HashMap::from(combos))
	}
}

impl<TSkill> Default for ComboNode<TSkill> {
	fn default() -> Self {
		Self(default())
	}
}

impl<TKey: Iterate<SlotKey>> Get<TKey, Skill> for ComboNode {
	fn get(&self, slot_key_path: &TKey) -> Option<&Skill> {
		let mut value = None;
		let mut combo_map = &self.0;

		for key in slot_key_path.iterate() {
			let (skill, node) = combo_map.get(key)?;
			value = Some(skill);
			combo_map = &node.0;
		}

		value
	}
}

impl<TKey: Iterate<SlotKey>> GetMut<TKey, Skill> for ComboNode {
	fn get_mut(&mut self, slot_key_path: &TKey) -> Option<&mut Skill> {
		let mut value = None;
		let mut combo_map = &mut self.0;

		for key in slot_key_path.iterate() {
			let (skill, node) = combo_map.get_mut(key)?;
			value = Some(skill);
			combo_map = &mut node.0;
		}

		value
	}
}

#[derive(Debug, PartialEq)]
pub enum SlotKeyPathError {
	IsEmpty,
	IsInvalid,
}

impl<TKey: Iterate<SlotKey>> TryInsert<TKey, Skill> for ComboNode {
	type Error = SlotKeyPathError;

	fn try_insert(&mut self, slot_key_path: TKey, value: Skill) -> Result<(), Self::Error> {
		let mut combo_map = &mut self.0;
		let mut slot_key_path = slot_key_path.iterate();
		let mut key = slot_key_path.next().ok_or(SlotKeyPathError::IsEmpty)?;

		for slot_key in slot_key_path {
			let (_, node) = combo_map.get_mut(key).ok_or(SlotKeyPathError::IsInvalid)?;
			combo_map = &mut node.0;
			key = slot_key;
		}

		match combo_map.entry(*key) {
			Entry::Occupied(mut entry) => {
				entry.get_mut().0 = value;
			}
			Entry::Vacant(entry) => {
				entry.insert((value, ComboNode::default()));
			}
		};

		Ok(())
	}
}

impl PeekNext<(Skill, ComboNode)> for ComboNode {
	fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<(Skill, ComboNode)> {
		let tree = &self.0;
		tree.get(trigger)
			.filter(|(skill, ..)| skill_is_usable(slots, trigger, skill))
			.cloned()
	}
}

fn skill_is_usable(slots: &Slots, trigger: &SlotKey, skill: &Skill) -> bool {
	let Some(slot) = slots.0.get(trigger) else {
		return false;
	};
	let Some(item) = slot.item.as_ref() else {
		return false;
	};
	skill
		.is_usable_with
		.intersection(&item.item_type)
		.next()
		.is_some()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Mounts, Slot},
		items::ItemType,
	};
	use bevy::{ecs::entity::Entity, prelude::default};
	use common::components::Side;
	use std::collections::HashSet;

	fn slots_main_pistol_off_sword() -> Slots {
		Slots(HashMap::from([
			(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: Mounts {
						hand: Entity::from_raw(123),
						forearm: Entity::from_raw(456),
					},
					item: Some(Item {
						item_type: HashSet::from([ItemType::Pistol]),
						..default()
					}),
				},
			),
			(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts {
						hand: Entity::from_raw(123),
						forearm: Entity::from_raw(456),
					},
					item: Some(Item {
						item_type: HashSet::from([ItemType::Bracer]),
						..default()
					}),
				},
			),
		]))
	}

	#[test]
	fn peek_next_from_tree() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(
			Some((
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)]))
			)),
			next
		)
	}

	#[test]
	fn peek_none_from_tree_when_slot_on_slot_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Off), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn peek_none_from_tree_when_slot_on_item_type_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Bracer]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn peek_none_from_tree_when_slot_item_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.get_mut(&SlotKey::Hand(Side::Main)).unwrap().item = None;

		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn peek_none_from_tree_when_slot_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.remove(&SlotKey::Hand(Side::Main));

		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_top_skill() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get(&[SlotKey::Hand(Side::Main)]);

		assert_eq!(
			Some(&Skill {
				name: "some skill".to_owned(),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn get_child_skill() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
			),
		)]);

		let skill = combos.get(&[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);

		assert_eq!(
			Some(&Skill {
				name: "some child skill".to_owned(),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn get_mut_top_skill() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get_mut(&[SlotKey::Hand(Side::Main)]).unwrap();
		*skill = Skill {
			name: "new skill".to_owned(),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "new skill".to_owned(),
						..default()
					},
					ComboNode::default(),
				),
			)]),
			combos
		);
	}

	#[test]
	fn get_mut_child_skill() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
			),
		)]);

		let skill = combos
			.get_mut(&[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)])
			.unwrap();
		*skill = Skill {
			name: "new skill".to_owned(),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "some skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "new skill".to_owned(),
								..default()
							},
							ComboNode::default(),
						),
					)]),
				),
			)]),
			combos
		);
	}

	#[test]
	fn try_insert_top_skill() {
		let mut combos = ComboNode::default();

		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "new skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
				Ok(())
			),
			(combos, success)
		);
	}

	#[test]
	fn try_insert_existing_skill_without_touching_child_skills() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "child skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
			),
		)]);

		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "new skill".to_owned(),
							..default()
						},
						ComboNode::new([(
							SlotKey::Hand(Side::Main),
							(
								Skill {
									name: "child skill".to_owned(),
									..default()
								},
								ComboNode::default(),
							),
						)]),
					),
				)]),
				Ok(())
			),
			(combos, success)
		);
	}

	#[test]
	fn try_insert_child_skill() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "some skill".to_owned(),
							..default()
						},
						ComboNode::new([(
							SlotKey::Hand(Side::Off),
							(
								Skill {
									name: "new skill".to_owned(),
									..default()
								},
								ComboNode::default(),
							),
						)]),
					),
				)]),
				Ok(())
			),
			(combos, success)
		);
	}

	#[test]
	fn error_when_provided_keys_empty() {
		let mut combos = ComboNode::default();
		let success = combos.try_insert(
			[],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(ComboNode::default(), Err(SlotKeyPathError::IsEmpty)),
			(combos, success)
		);
	}

	#[test]
	fn error_when_provided_keys_can_not_be_followed() {
		let mut combos = ComboNode::default();
		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(ComboNode::default(), Err(SlotKeyPathError::IsInvalid)),
			(combos, success)
		);
	}
}
