use super::{combo_node::ComboNode, Slots};
use crate::{
	items::slot_key::SlotKey,
	skills::Skill,
	traits::{
		Combo,
		GetCombos,
		GetEntry,
		GetEntryMut,
		Insert,
		PeekNext,
		ReKey,
		SetNextCombo,
		UpdateConfig,
	},
};
use bevy::ecs::component::Component;
use common::traits::iterate::Iterate;

#[derive(Component, PartialEq, Debug)]
pub struct Combos<TComboNode = ComboNode> {
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

impl<T> From<ComboNode<T>> for Combos<ComboNode<T>> {
	fn from(value: ComboNode<T>) -> Self {
		Combos::new(value)
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

impl<TNode: GetCombos> GetCombos for Combos<TNode> {
	fn combos(&self) -> Vec<Combo> {
		self.value.combos()
	}
}

impl<TNode, TKey> UpdateConfig<TKey, Option<Skill>> for Combos<TNode>
where
	for<'a> TNode: GetEntryMut<'a, TKey, TEntry: Insert<Option<Skill>>>,
	TKey: Iterate<SlotKey>,
{
	fn update_config(&mut self, key_path: &TKey, skill: Option<Skill>) {
		self.current = None;

		let Some(mut entry) = self.value.entry_mut(key_path) else {
			return;
		};

		entry.insert(skill);
	}
}

impl<TNode, TKey> UpdateConfig<TKey, SlotKey> for Combos<TNode>
where
	for<'a> TNode: GetEntryMut<'a, TKey, TEntry: ReKey<SlotKey>>,
	TKey: Iterate<SlotKey>,
{
	fn update_config(&mut self, key_path: &TKey, key: SlotKey) {
		self.current = None;

		let Some(mut entry) = self.value.entry_mut(key_path) else {
			return;
		};

		entry.re_key(key);
	}
}

impl<'a, TNode: GetEntry<'a, TKey>, TKey: Iterate<SlotKey>> GetEntry<'a, TKey> for Combos<TNode> {
	type TEntry = TNode::TEntry;

	fn entry(&'a self, key: &TKey) -> Option<Self::TEntry> {
		self.value.entry(key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Mounts, Slot};
	use bevy::{ecs::entity::Entity, utils::default};
	use common::components::Side;
	use mockall::{mock, predicate::eq};
	use std::{cell::RefCell, collections::HashMap};

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
					name: "my skill".to_owned(),
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
				name: "my skill".to_owned(),
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

	struct _ComboNode<'a>(Vec<Combo<'a>>);

	impl<'a> GetCombos for _ComboNode<'a> {
		fn combos(&self) -> Vec<Combo> {
			self.0.clone()
		}
	}

	#[test]
	fn get_combos_from_config() {
		let skill = Skill {
			name: "my skill".to_owned(),
			..default()
		};
		let combos_vec = vec![vec![(
			vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			&skill,
		)]];
		let combos = Combos::new(_ComboNode(combos_vec.clone()));

		assert_eq!(combos_vec, combos.combos())
	}

	#[derive(Default)]
	struct _Node {
		call_args: RefCell<Vec<Vec<SlotKey>>>,
		entry: Option<_Entry>,
	}

	impl<'a> GetEntryMut<'a, Vec<SlotKey>> for _Node {
		type TEntry = &'a mut _Entry;

		fn entry_mut(&'a mut self, key: &Vec<SlotKey>) -> Option<Self::TEntry> {
			self.call_args.get_mut().push(key.clone());
			self.entry.as_mut()
		}
	}

	impl<'a> GetEntry<'a, Vec<SlotKey>> for _Node {
		type TEntry = &'a _Entry;

		fn entry(&'a self, key: &Vec<SlotKey>) -> Option<Self::TEntry> {
			self.call_args.borrow_mut().push(key.clone());
			self.entry.as_ref()
		}
	}

	mock! {
		_Entry {}
		impl Insert<Option<Skill>> for _Entry {
			fn insert(&mut self, value: Option<Skill>);
		}
		impl ReKey<SlotKey> for _Entry {
			fn re_key(&mut self, key: SlotKey);
		}
	}

	#[derive(Default)]
	struct _Entry {
		mock: Mock_Entry,
	}

	impl<'a> Insert<Option<Skill>> for &'a mut _Entry {
		fn insert(&mut self, value: Option<Skill>) {
			self.mock.insert(value)
		}
	}

	impl<'a> ReKey<SlotKey> for &'a mut _Entry {
		fn re_key(&mut self, key: SlotKey) {
			self.mock.re_key(key)
		}
	}

	#[test]
	fn update_config_skill_use_correct_arguments() {
		let mut entry = _Entry::default();
		entry.mock.expect_insert().return_const(());

		let mut combos = Combos::new(_Node {
			entry: Some(entry),
			..default()
		});

		combos.update_config(
			&vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			Some(Skill {
				name: "my skill".to_owned(),
				..default()
			}),
		);

		assert_eq!(
			vec![vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]],
			combos.value.call_args.into_inner()
		)
	}

	#[test]
	fn update_config_skill_call_entry_insert() {
		let mut entry = _Entry::default();
		entry
			.mock
			.expect_insert()
			.times(1)
			.with(eq(Some(Skill {
				name: "my skill".to_owned(),
				..default()
			})))
			.return_const(());

		let mut combos = Combos::new(_Node {
			entry: Some(entry),
			..default()
		});

		combos.update_config(
			&vec![SlotKey::Hand(Side::Main)],
			Some(Skill {
				name: "my skill".to_owned(),
				..default()
			}),
		);
	}

	#[test]
	fn update_config_skill_clear_current() {
		let mut entry = _Entry::default();
		entry.mock.expect_insert().return_const(());

		let mut combos = Combos {
			value: _Node {
				entry: Some(entry),
				..default()
			},
			current: Some(_Node::default()),
		};

		combos.update_config(
			&vec![SlotKey::Hand(Side::Main)],
			Some(Skill {
				name: "my skill".to_owned(),
				..default()
			}),
		);

		assert!(combos.current.is_none());
	}

	#[test]
	fn update_config_skill_clear_current_if_entry_is_none() {
		let mut combos = Combos {
			value: _Node {
				entry: None,
				..default()
			},
			current: Some(_Node::default()),
		};

		combos.update_config(
			&vec![SlotKey::Hand(Side::Main)],
			Some(Skill {
				name: "my skill".to_owned(),
				..default()
			}),
		);

		assert!(combos.current.is_none());
	}

	#[test]
	fn update_config_re_key_use_correct_arguments() {
		let mut entry = _Entry::default();
		entry.mock.expect_re_key().return_const(());

		let mut combos = Combos::new(_Node {
			entry: Some(entry),
			..default()
		});

		combos.update_config(
			&vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			SlotKey::Hand(Side::Off),
		);

		assert_eq!(
			vec![vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]],
			combos.value.call_args.into_inner()
		)
	}

	#[test]
	fn update_config_re_key_call_entry_re_key() {
		let mut entry = _Entry::default();
		entry
			.mock
			.expect_re_key()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Off)))
			.return_const(());

		let mut combos = Combos::new(_Node {
			entry: Some(entry),
			..default()
		});

		combos.update_config(&vec![SlotKey::Hand(Side::Main)], SlotKey::Hand(Side::Off));
	}

	#[test]
	fn update_config_re_key_clear_current() {
		let mut entry = _Entry::default();
		entry.mock.expect_re_key().return_const(());

		let mut combos = Combos {
			value: _Node {
				entry: Some(entry),
				..default()
			},
			current: Some(_Node::default()),
		};

		combos.update_config(&vec![SlotKey::Hand(Side::Main)], SlotKey::Hand(Side::Off));

		assert!(combos.current.is_none());
	}

	#[test]
	fn update_config_re_key_clear_current_if_entry_is_none() {
		let mut combos = Combos {
			value: _Node {
				entry: None,
				..default()
			},
			current: Some(_Node::default()),
		};

		combos.update_config(&vec![SlotKey::Hand(Side::Main)], SlotKey::Hand(Side::Off));

		assert!(combos.current.is_none());
	}

	#[test]
	fn get_entry() {
		let combo = Combos {
			value: _Node {
				entry: Some(_Entry::default()),
				..default()
			},
			current: None,
		};

		let entry = combo.entry(&vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);

		assert_eq!(
			(
				true,
				vec![vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]]
			),
			(entry.is_some(), combo.value.call_args.into_inner())
		)
	}
}
