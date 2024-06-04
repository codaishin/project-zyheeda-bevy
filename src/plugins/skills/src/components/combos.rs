use super::{SlotKey, Slots};
use crate::{
	skills::Skill,
	traits::{PeekNext, SetNextCombo},
};
use bevy::ecs::component::Component;
use common::traits::{get::GetMut, insert::TryInsert, iterate::Iterate};
use std::collections::{hash_map::Entry, HashMap};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ComboNode(HashMap<SlotKey, (Skill, ComboNode)>);

impl ComboNode {
	pub fn new<const N: usize>(combos: [(SlotKey, (Skill, ComboNode)); N]) -> Self {
		Self(HashMap::from(combos))
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

#[derive(Component)]
pub struct Combos<TComboNode = ComboNode> {
	value: TComboNode,
	current: Option<TComboNode>,
}

impl<T> Combos<T> {
	#[allow(dead_code)]
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
			value: ComboNode::default(),
			current: None,
		}
	}
}

impl<TComboNode> SetNextCombo<Option<TComboNode>> for Combos<TComboNode> {
	fn set_next_combo(&mut self, value: Option<TComboNode>) {
		self.current = value;
	}
}

impl<TComboNode: PeekNext<(Skill, TComboNode)>> PeekNext<(Skill, TComboNode)>
	for Combos<TComboNode>
{
	fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<(Skill, TComboNode)> {
		self.current
			.as_ref()
			.and_then(|current| current.peek_next(trigger, slots))
			.or_else(|| self.value.peek_next(trigger, slots))
	}
}

#[cfg(test)]
mod test_combo_node {
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
	fn get_next_from_tree() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
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
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
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
	fn get_none_from_tree_when_slot_on_slot_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
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
	fn get_none_from_tree_when_slot_on_item_type_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Bracer]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
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
	fn get_none_from_tree_when_slot_item_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.get_mut(&SlotKey::Hand(Side::Main)).unwrap().item = None;

		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
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
	fn get_none_from_tree_when_slot_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.remove(&SlotKey::Hand(Side::Main));

		let node = ComboNode(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first",
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(HashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second",
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
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill",
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get_mut(&[SlotKey::Hand(Side::Main)]).unwrap();
		*skill = Skill {
			name: "new skill",
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "new skill",
						..default()
					},
					ComboNode::default(),
				),
			)]),
			combos
		);
	}

	#[test]
	fn get_child_skill() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill",
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "some child skill",
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
			name: "new skill",
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "some skill",
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "new skill",
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
				name: "new skill",
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "new skill",
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
					name: "some skill",
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "child skill",
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
				name: "new skill",
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "new skill",
							..default()
						},
						ComboNode::new([(
							SlotKey::Hand(Side::Main),
							(
								Skill {
									name: "child skill",
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
					name: "some skill",
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			Skill {
				name: "new skill",
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "some skill",
							..default()
						},
						ComboNode::new([(
							SlotKey::Hand(Side::Off),
							(
								Skill {
									name: "new skill",
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
				name: "new skill",
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
				name: "new skill",
				..default()
			},
		);

		assert_eq!(
			(ComboNode::default(), Err(SlotKeyPathError::IsInvalid)),
			(combos, success)
		);
	}
}

#[cfg(test)]
mod test_combos {
	use super::*;
	use crate::components::{Mounts, Slot};
	use bevy::{ecs::entity::Entity, utils::default};
	use common::components::Side;
	use mockall::{mock, predicate::eq};

	mock! {
		_Next {}
		impl PeekNext<(Skill, Self)> for _Next {
			fn peek_next(&self, _trigger: &SlotKey, _slots: &Slots) -> Option<(Skill, Self)>;
		}
	}

	#[test]
	fn call_next_with_correct_args() {
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: Mounts {
					hand: Entity::from_raw(123),
					forearm: Entity::from_raw(456),
				},
				item: None,
			},
		)]));
		let trigger = SlotKey::Hand(Side::Off);

		let mut mock = Mock_Next::default();
		mock.expect_peek_next()
			.times(1)
			.with(eq(trigger), eq(slots.clone()))
			.returning(|_, _| None);

		let combos = Combos::new(mock);

		combos.peek_next(&trigger, &slots);
	}

	#[test]
	fn return_skill() {
		let mut mock = Mock_Next::default();
		mock.expect_peek_next().returning(|_, _| {
			Some((
				Skill {
					name: "my skill",
					..default()
				},
				Mock_Next::default(),
			))
		});
		let combos = Combos::new(mock);

		let skill = combos
			.peek_next(&default(), &default())
			.map(|(skill, _)| skill);

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
		mock.expect_peek_next().returning(|_, _| None);
		let combos = Combos::new(mock);

		let skill = combos.peek_next(&default(), &default());

		assert!(skill.is_none());
	}

	#[test]
	fn return_next_node() {
		#[derive(Debug, PartialEq)]
		struct _Node(&'static str);

		impl PeekNext<(Skill, _Node)> for _Node {
			fn peek_next(&self, _: &SlotKey, _: &Slots) -> Option<(Skill, _Node)> {
				Some((Skill::default(), _Node("next")))
			}
		}

		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: Mounts {
					hand: Entity::from_raw(123),
					forearm: Entity::from_raw(456),
				},
				item: None,
			},
		)]));
		let trigger = SlotKey::Hand(Side::Off);

		let combos = Combos::new(_Node("first"));

		let next_combo = combos.peek_next(&trigger, &slots).map(|(_, node)| node);

		assert_eq!(Some(_Node("next")), next_combo);
	}

	#[test]
	fn use_combo_used_in_set_next_combo() {
		let mut first = Mock_Next::default();
		let mut next = Mock_Next::default();

		first.expect_peek_next().never().returning(|_, _| None);
		next.expect_peek_next()
			.times(1)
			.returning(|_, _| Some((Skill::default(), Mock_Next::default())));

		let mut combos = Combos::new(first);

		combos.set_next_combo(Some(next));
		combos.peek_next(&default(), &default());
	}

	#[test]
	fn use_original_when_next_combo_returns_none() {
		let mut first = Mock_Next::default();
		let mut other = Mock_Next::default();

		first.expect_peek_next().times(1).returning(|_, _| None);
		other.expect_peek_next().returning(|_, _| None);

		let mut combos = Combos::new(first);

		combos.set_next_combo(Some(other));
		combos.peek_next(&default(), &default());
	}

	#[test]
	fn use_original_when_set_next_combo_with_none() {
		let mut first = Mock_Next::default();
		let mut other = Mock_Next::default();

		first.expect_peek_next().times(1).returning(|_, _| None);
		other
			.expect_peek_next()
			.never()
			.returning(|_, _| Some((Skill::default(), Mock_Next::default())));

		let mut combos = Combos::new(first);

		combos.set_next_combo(Some(other));
		combos.set_next_combo(None);
		combos.peek_next(&default(), &default());
	}
}
