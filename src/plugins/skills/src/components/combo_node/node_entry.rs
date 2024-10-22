use super::{ComboNode, NodeEntry};
use crate::{slot_key::SlotKey, traits::FollowupKeys};

#[derive(Default)]
enum Iter<'a, T: Iterator<Item = &'a SlotKey>> {
	#[default]
	None,
	Some(T),
}

impl<'a, T: Iterator<Item = &'a SlotKey>> Iterator for Iter<'a, T> {
	type Item = SlotKey;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Iter::None => None,
			Iter::Some(keys) => keys.next().cloned(),
		}
	}
}

impl<'a, TSkill> FollowupKeys for NodeEntry<'a, TSkill> {
	type TItem = SlotKey;

	fn followup_keys(&self) -> impl Iterator<Item = Self::TItem> {
		let Some((_, ComboNode(followup_tree))) = self.tree.get(&self.key) else {
			return Iter::None;
		};

		Iter::Some(followup_tree.keys())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::combo_node::ComboNode;
	use bevy::prelude::default;
	use common::{components::Side, tools::ordered_hash_map::OrderedHashMap};
	use std::collections::HashSet;

	struct _Skill;

	#[test]
	fn no_followup_keys_when_no_skill_follows() {
		let entry = NodeEntry::<_Skill> {
			key: SlotKey::BottomHand(Side::Right),
			tree: &OrderedHashMap::from([(SlotKey::BottomHand(Side::Right), (_Skill, default()))]),
		};

		assert_eq!(
			vec![] as Vec<SlotKey>,
			entry.followup_keys().collect::<Vec<_>>()
		);
	}

	#[test]
	fn iterate_followup_keys() {
		let entry = NodeEntry::<_Skill> {
			key: SlotKey::BottomHand(Side::Right),
			tree: &OrderedHashMap::from([(
				SlotKey::BottomHand(Side::Right),
				(
					_Skill,
					ComboNode::new([
						(SlotKey::BottomHand(Side::Right), (_Skill, default())),
						(SlotKey::BottomHand(Side::Left), (_Skill, default())),
					]),
				),
			)]),
		};

		assert_eq!(
			HashSet::from([
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left)
			]),
			entry.followup_keys().collect::<HashSet<_>>()
		);
	}

	#[test]
	fn no_followup_keys_when_entry_empty() {
		let entry = NodeEntry::<_Skill> {
			key: SlotKey::BottomHand(Side::Right),
			tree: &OrderedHashMap::from([]),
		};

		assert_eq!(
			vec![] as Vec<SlotKey>,
			entry.followup_keys().collect::<Vec<_>>()
		);
	}
}
