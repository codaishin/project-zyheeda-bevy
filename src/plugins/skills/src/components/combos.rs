use super::{SlotKey, Slots};
use crate::{
	skill::{Queued, Skill},
	traits::NextCombo,
};
use bevy::{ecs::component::Component, prelude::default};
use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
pub enum ComboNode {
	Tree(HashMap<SlotKey, (Skill, ComboNode)>),
	Circle(VecDeque<(SlotKey, Skill)>),
}

#[derive(Component)]
pub(crate) struct Combos {
	value: ComboNode,
	current: Option<ComboNode>,
}

impl Combos {
	pub fn new(config: ComboNode) -> Self {
		Self {
			value: config,
			current: None,
		}
	}
}

impl Default for Combos {
	fn default() -> Self {
		Self {
			value: ComboNode::Tree(default()),
			current: None,
		}
	}
}

impl NextCombo for Combos {
	fn next(&mut self, trigger_skill: &Skill<Queued>, slots: &Slots) -> Option<Skill> {
		let slot_key = &trigger_skill.data.slot_key;
		let Some((skill, next_combo_iteration)) = combo(self, slot_key) else {
			self.current = None;
			return None;
		};

		if is_not_usable(&skill, slot_key, slots) {
			self.current = None;
			return None;
		}

		if combo_empty(&next_combo_iteration) {
			self.current = None;
		} else {
			self.current = Some(next_combo_iteration);
		}

		Some(skill.clone())
	}
}

fn combo_empty(combos: &ComboNode) -> bool {
	match combos {
		ComboNode::Tree(tree) => tree.is_empty(),
		ComboNode::Circle(buffer) => buffer.is_empty(),
	}
}

fn combo(combos: &mut Combos, slot_key: &SlotKey) -> Option<(Skill, ComboNode)> {
	match combos.current.as_ref().unwrap_or(&combos.value) {
		ComboNode::Tree(tree) => tree.get(slot_key).cloned(),
		ComboNode::Circle(circle) => get_skill_and_circle(circle, slot_key),
	}
}

fn get_skill_and_circle(
	buffer: &VecDeque<(SlotKey, Skill)>,
	slot_key: &SlotKey,
) -> Option<(Skill, ComboNode)> {
	let mut buffer = buffer.clone();

	let Some((key, skill)) = buffer.pop_front() else {
		return None;
	};

	if &key != slot_key {
		return None;
	}

	buffer.push_back((key, skill.clone()));
	Some((skill, ComboNode::Circle(buffer)))
}

fn is_not_usable(skill: &Skill, slot_key: &SlotKey, slots: &Slots) -> bool {
	let Some(slot) = slots.0.get(slot_key) else {
		return true;
	};
	let Some(item) = slot.item.as_ref() else {
		return true;
	};

	item.item_type.is_disjoint(&skill.is_usable_with)
}

#[cfg(test)]
mod test_combos {
	use super::*;
	use crate::components::{Item, ItemType, Slot};
	use bevy::{ecs::entity::Entity, utils::default};
	use common::components::Side;
	use std::collections::HashSet;

	#[test]
	fn get_next_from_tree() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "combo skill",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(default()),
			),
		)])));

		let next = combos.next(&skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next
		);
	}

	#[test]
	fn do_not_get_next_from_tree_when_item_type_mismatch() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Sword]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "combo skill",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(default()),
			),
		)])));

		let next = combos.next(&skill, &slots);

		assert_eq!(None, next);
	}

	#[test]
	fn get_next_next_from_tree() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "combo skill",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "combo skill b",
							is_usable_with: HashSet::from([ItemType::Pistol]),
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)])));

		_ = combos.next(&skill, &slots);
		let next_next = combos.next(&skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill b",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next_next
		);
	}

	#[test]
	fn reset_combo_on_when_combo_dropped() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "combo skill b",
							is_usable_with: HashSet::from([ItemType::Sword]),
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)])));

		_ = combos.next(&skill, &slots);
		_ = combos.next(&skill, &slots);
		let next = combos.next(&skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill a",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next
		);
	}

	#[test]
	fn reset_combo_on_item_type_mismatch() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "combo skill b",
							is_usable_with: HashSet::from([ItemType::Sword]),
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)])));

		_ = combos.next(&skill, &slots);
		_ = combos.next(&skill, &slots);
		let next = combos.next(&skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill a",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next
		);
	}

	#[test]
	fn reset_combo_when_slot_item_none() {
		let slots = Slots(HashMap::from([
			(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(123),
					item: Some(Item {
						item_type: HashSet::from([ItemType::Pistol]),
						..default()
					}),
				},
			),
			(
				SlotKey::Hand(Side::Off),
				Slot {
					entity: Entity::from_raw(123),
					item: None,
				},
			),
		]));
		let main_skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let off_skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "combo skill b",
							is_usable_with: HashSet::from([ItemType::Pistol]),
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)])));

		_ = combos.next(&main_skill, &slots);
		_ = combos.next(&off_skill, &slots);
		let next = combos.next(&main_skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill a",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next
		);
	}

	#[test]
	fn reset_combo_when_slot_does_not_exist() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let main_skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let off_skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "combo skill b",
							is_usable_with: HashSet::from([ItemType::Pistol]),
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)])));

		_ = combos.next(&main_skill, &slots);
		_ = combos.next(&off_skill, &slots);
		let next = combos.next(&main_skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill a",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next
		);
	}

	#[test]
	fn reset_combo_when_combo_done() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "combo skill b",
							is_usable_with: HashSet::from([ItemType::Pistol]),
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)])));

		_ = combos.next(&skill, &slots);
		_ = combos.next(&skill, &slots);
		let next = combos.next(&skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill a",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next
		);
	}

	#[test]
	fn get_next_from_circle() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Circle(VecDeque::from([(
			SlotKey::Hand(Side::Main),
			Skill {
				name: "combo skill",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			},
		)])));

		let next = combos.next(&skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next
		);
	}

	#[test]
	fn get_next_next_from_circle() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Circle(VecDeque::from([
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "combo skill b",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
		])));

		_ = combos.next(&skill, &slots);
		let next_next = combos.next(&skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill b",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			next_next
		);
	}

	#[test]
	fn get_first_after_full_circle() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Circle(VecDeque::from([
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "combo skill b",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
		])));

		_ = combos.next(&skill, &slots);
		_ = combos.next(&skill, &slots);
		let first = combos.next(&skill, &slots);

		assert_eq!(
			Some(Skill {
				name: "combo skill a",
				is_usable_with: HashSet::from([ItemType::Pistol]),
				..default()
			}),
			first
		);
	}

	#[test]
	fn reset_when_circle_combo_dropped() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(123),
				item: Some(Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				}),
			},
		)]));
		let skill = Skill {
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let mut combos = Combos::new(ComboNode::Circle(VecDeque::from([
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Off),
				Skill {
					name: "combo skill b",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Off),
				Skill {
					name: "additional skill to prevent combo from being done",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
		])));

		_ = combos.next(&skill, &slots);
		let dropped = combos.next(&skill, &slots);
		let first = combos.next(&skill, &slots);

		assert_eq!(
			(
				None,
				Some(Skill {
					name: "combo skill a",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				})
			),
			(dropped, first)
		);
	}
}
