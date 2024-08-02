use super::{ComboNode, NodeEntry};
use crate::{items::slot_key::SlotKey, traits::FollowupKeys};
use std::collections::hash_map::Keys;

#[derive(Default)]
enum Iter<'a, TSkill: 'a> {
	#[default]
	None,
	Some(Keys<'a, SlotKey, TSkill>),
}

impl<'a, TSkill> Iterator for Iter<'a, TSkill> {
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
	use common::components::Side;
	use std::collections::{HashMap, HashSet};

	struct _Skill;

	#[test]
	fn no_followup_keys_when_no_skill_follows() {
		let entry = NodeEntry::<_Skill> {
			key: SlotKey::Hand(Side::Main),
			tree: &HashMap::from([(SlotKey::Hand(Side::Main), (_Skill, default()))]),
		};

		assert_eq!(
			vec![] as Vec<SlotKey>,
			entry.followup_keys().collect::<Vec<_>>()
		);
	}

	#[test]
	fn iterate_followup_keys() {
		let entry = NodeEntry::<_Skill> {
			key: SlotKey::Hand(Side::Main),
			tree: &HashMap::from([(
				SlotKey::Hand(Side::Main),
				(
					_Skill,
					ComboNode::new([
						(SlotKey::Hand(Side::Main), (_Skill, default())),
						(SlotKey::Hand(Side::Off), (_Skill, default())),
					]),
				),
			)]),
		};

		assert_eq!(
			HashSet::from([SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]),
			entry.followup_keys().collect::<HashSet<_>>()
		);
	}

	#[test]
	fn no_followup_keys_when_entry_empty() {
		let entry = NodeEntry::<_Skill> {
			key: SlotKey::Hand(Side::Main),
			tree: &HashMap::from([]),
		};

		assert_eq!(
			vec![] as Vec<SlotKey>,
			entry.followup_keys().collect::<Vec<_>>()
		);
	}
}
