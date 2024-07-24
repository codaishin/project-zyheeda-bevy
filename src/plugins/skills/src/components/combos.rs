use super::{combo_node::ComboNode, Slots};
use crate::{
	items::slot_key::SlotKey,
	skills::Skill,
	traits::{Combo, GetCombos, PeekNext, SetNextCombo, UpdateConfig},
};
use bevy::ecs::component::Component;
use common::traits::{get::GetMut, iterate::Iterate};

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

impl<TNode, TKey> UpdateConfig<TKey> for Combos<TNode>
where
	TNode: GetMut<TKey, Skill>,
	TKey: Iterate<SlotKey>,
{
	fn update_config(&mut self, key: &TKey, skill: Skill) {
		self.current = None;

		let Some(node_skill) = self.value.get_mut(key) else {
			return;
		};
		*node_skill = skill
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Mounts, Slot};
	use bevy::{ecs::entity::Entity, utils::default};
	use common::components::Side;
	use mockall::{automock, mock, predicate::eq};
	use std::{collections::HashMap, marker::PhantomData};

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
	struct _Node<TKey>(PhantomData<TKey>);

	#[automock]
	impl<TKey> GetMut<TKey, Skill> for _Node<TKey> {
		fn get_mut<'a>(&'a mut self, _key: &TKey) -> Option<&'a mut Skill> {
			None
		}
	}

	#[test]
	fn update_config_get_node_skill_through_correct_args() {
		let mut get_mut = Mock_Node::<Vec<SlotKey>>::default();
		get_mut
			.expect_get_mut()
			.times(1)
			.with(eq(vec![
				SlotKey::Hand(Side::Off),
				SlotKey::Hand(Side::Main),
			]))
			.return_once(|_| None);
		let mut combos = Combos::new(get_mut);

		combos.update_config(
			&vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			Skill::default(),
		);
	}

	macro_rules! make_static {
		($value:expr) => {
			static mut VALUES: Vec<Skill> = vec![];
			unsafe { VALUES.push($value) }

			fn get_static_skill() -> Option<&'static mut Skill> {
				unsafe { VALUES.get_mut(0) }
			}
		};
	}

	#[test]
	fn update_config_set_node_skill() {
		make_static!(Skill {
			name: "my skill".to_owned(),
			..default()
		});

		let mut get_mut = Mock_Node::<Vec<SlotKey>>::default();
		get_mut.expect_get_mut().return_once(|_| get_static_skill());
		let mut combos = Combos::new(get_mut);

		combos.update_config(
			&vec![],
			Skill {
				name: "my other skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			Some(&mut Skill {
				name: "my other skill".to_owned(),
				..default()
			}),
			get_static_skill()
		);
	}

	#[test]
	fn update_config_reset_current_combos() {
		let mut combos = Combos {
			value: _Node::<Vec<SlotKey>>::default(),
			current: Some(_Node::<Vec<SlotKey>>::default()),
		};

		combos.update_config(&vec![], Skill::default());

		assert!(combos.current.is_none());
	}
}
