use super::{SlotKey, Slots};
use crate::{
	skill::Skill,
	traits::{Flush, NextCombo},
};
use bevy::{ecs::component::Component, prelude::default};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, PartialEq, Debug)]
pub enum ComboNode {
	Tree(HashMap<SlotKey, (Skill, ComboNode)>),
	Circle(VecDeque<(SlotKey, Skill)>),
}

trait GetNext
where
	Self: Sized,
{
	fn next(&self, trigger: &SlotKey, slots: &Slots) -> Option<(Skill, Self)>;
}

impl GetNext for ComboNode {
	fn next(&self, trigger: &SlotKey, slots: &Slots) -> Option<(Skill, Self)> {
		match self {
			ComboNode::Tree(tree) => tree
				.get(trigger)
				.filter(|(skill, ..)| skill_is_usable(slots, trigger, skill))
				.cloned(),
			ComboNode::Circle(circle) => circle
				.front()
				.filter(|(key, ..)| key == trigger)
				.filter(|(.., skill)| skill_is_usable(slots, trigger, skill))
				.map(|(.., s)| (s.clone(), rotated(circle))),
		}
	}
}

fn rotated(circle: &VecDeque<(SlotKey, Skill)>) -> ComboNode {
	let mut circle = circle.clone();
	if let Some(front) = circle.pop_front() {
		circle.push_back(front)
	}
	ComboNode::Circle(circle)
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

#[derive(Component)]
pub(crate) struct Combos<TComboNode = ComboNode> {
	value: TComboNode,
	current: Option<TComboNode>,
}

impl<T> Combos<T> {
	pub fn new(config: T) -> Self {
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

impl<TComboNode: GetNext> NextCombo for Combos<TComboNode> {
	fn next(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {
		let Some((skill, next)) = self
			.current
			.as_ref()
			.unwrap_or(&self.value)
			.next(trigger, slots)
		else {
			self.current = None;
			return None;
		};
		self.current = Some(next);
		Some(skill)
	}
}

impl<T> Flush for Combos<T> {
	fn flush(&mut self) {
		self.current = None;
	}
}

#[cfg(test)]
mod test_combo_node {
	use super::*;
	use crate::components::{Item, ItemType, Slot};
	use bevy::ecs::entity::Entity;
	use common::components::Side;
	use std::collections::HashSet;

	fn slots_main_pistol_off_sword() -> Slots {
		Slots(HashMap::from([
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
					item: Some(Item {
						item_type: HashSet::from([ItemType::Sword]),
						..default()
					}),
				},
			),
		]))
	}

	#[test]
	fn get_next_from_tree() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)]));

		let next = node.next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(
			Some((
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
							..default()
						},
						ComboNode::Tree(default()),
					),
				)]))
			)),
			next
		)
	}

	#[test]
	fn get_none_from_tree_when_slot_on_slot_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)]));

		let next = node.next(&SlotKey::Hand(Side::Off), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_none_from_tree_when_slot_on_item_type_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Sword]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)]));

		let next = node.next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_none_from_tree_when_slot_item_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.get_mut(&SlotKey::Hand(Side::Main)).unwrap().item = None;

		let node = ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)]));

		let next = node.next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_none_from_tree_when_slot_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.remove(&SlotKey::Hand(Side::Main));

		let node = ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Tree(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
							..default()
						},
						ComboNode::Tree(default()),
					),
				)])),
			),
		)]));

		let next = node.next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_next_from_circle() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode::Circle(VecDeque::from([
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "second",
					..default()
				},
			),
		]));

		let next = node.next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(
			Some((
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode::Circle(VecDeque::from([
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "second",
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "first",
							is_usable_with: HashSet::from([ItemType::Pistol]),
							..default()
						},
					),
				]))
			)),
			next
		)
	}

	#[test]
	fn get_none_from_circle_when_slot_on_slot_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode::Circle(VecDeque::from([
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Sword]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "second",
					..default()
				},
			),
		]));

		let next = node.next(&SlotKey::Hand(Side::Off), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_none_from_circle_when_slot_on_item_type_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode::Circle(VecDeque::from([
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Sword]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "second",
					..default()
				},
			),
		]));

		let next = node.next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_none_from_circle_when_slot_item_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.get_mut(&SlotKey::Hand(Side::Main)).unwrap().item = None;

		let node = ComboNode::Circle(VecDeque::from([
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "second",
					..default()
				},
			),
		]));

		let next = node.next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_none_from_circle_when_slot_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.remove(&SlotKey::Hand(Side::Main));

		let node = ComboNode::Circle(VecDeque::from([
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
			),
			(
				SlotKey::Hand(Side::Main),
				Skill {
					name: "second",
					..default()
				},
			),
		]));

		let next = node.next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}
}

#[cfg(test)]
mod test_combos {
	use super::*;
	use crate::components::Slot;
	use bevy::{ecs::entity::Entity, utils::default};
	use common::components::Side;
	use mockall::{mock, predicate::eq};

	mock! {
		_Next {}
		impl GetNext for _Next {
			fn next(&self, _trigger: &SlotKey, _slots: &Slots) -> Option<(Skill, Self)>;
		}
	}

	#[test]
	fn call_next_with_correct_args() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(1234),
				item: None,
			},
		)]));
		let trigger = SlotKey::Hand(Side::Off);

		let mut mock = Mock_Next::default();
		mock.expect_next()
			.times(1)
			.with(eq(trigger), eq(slots.clone()))
			.returning(|_, _| None);

		let mut combos = Combos::new(mock);

		combos.next(&trigger, &slots);
	}

	#[test]
	fn return_skill() {
		let mut mock = Mock_Next::default();
		mock.expect_next().returning(|_, _| {
			Some((
				Skill {
					name: "my skill",
					..default()
				},
				Mock_Next::default(),
			))
		});
		let mut combos = Combos::new(mock);

		let skill = combos.next(&default(), &default());

		assert_eq!(
			Some(Skill {
				name: "my skill",
				..default()
			}),
			skill
		);
	}

	#[test]
	fn return_none() {
		let mut mock = Mock_Next::default();
		mock.expect_next().returning(|_, _| None);
		let mut combos = Combos::new(mock);

		let skill = combos.next(&default(), &default());

		assert_eq!(None, skill);
	}

	#[test]
	fn use_subsequent_next_calls() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(1234),
				item: None,
			},
		)]));
		let slots_clone = slots.clone();
		let trigger = SlotKey::Hand(Side::Off);

		let mut mock = Mock_Next::default();
		mock.expect_next().times(1).returning(move |_, _| {
			let mut mock = Mock_Next::default();
			mock.expect_next()
				.times(1)
				.with(eq(trigger), eq(slots_clone.clone()))
				.returning(|_, _| None);
			Some((Skill::default(), mock))
		});

		let mut combos = Combos::new(mock);

		combos.next(&trigger, &slots);
		combos.next(&trigger, &slots);
	}

	#[test]
	fn reset_to_use_top_next_after_none_return() {
		let mut mock = Mock_Next::default();
		mock.expect_next().times(2).returning(move |_, _| {
			let mut mock = Mock_Next::default();
			mock.expect_next().times(1).returning(|_, _| None);
			Some((Skill::default(), mock))
		});

		let mut combos = Combos::new(mock);

		combos.next(&default(), &default()); // 1st top call
		combos.next(&default(), &default()); // new subsequent called once
		combos.next(&default(), &default()); // 2nd top call
		combos.next(&default(), &default()); // new subsequent called once
	}

	#[test]
	fn reset_to_use_top_next_after_flush() {
		let mut mock = Mock_Next::default();
		mock.expect_next().times(2).returning(move |_, _| {
			let mut mock = Mock_Next::default();
			mock.expect_next().never().returning(|_, _| None);
			Some((Skill::default(), mock))
		});

		let mut combos = Combos::new(mock);

		combos.next(&default(), &default());
		combos.flush();
		combos.next(&default(), &default());
	}
}
