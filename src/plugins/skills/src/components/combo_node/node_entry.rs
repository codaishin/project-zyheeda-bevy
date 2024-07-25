use super::NodeEntry;
use crate::traits::Insert;
use bevy::prelude::default;
use std::collections::hash_map::Entry;

impl<'a, TSkill> Insert<Option<TSkill>> for NodeEntry<'a, TSkill> {
	fn insert(&mut self, value: Option<TSkill>) {
		match value {
			Some(value) => update_entry(self, value),
			None => clear_entry(self),
		}
	}
}

fn update_entry<TSkill>(entry: &mut NodeEntry<TSkill>, value: TSkill) {
	match entry.tree.entry(entry.key) {
		Entry::Occupied(mut entry) => {
			let (skill, _) = entry.get_mut();
			*skill = value;
		}
		Entry::Vacant(entry) => {
			entry.insert((value, default()));
		}
	}
}

fn clear_entry<TSkill>(entry: &mut NodeEntry<TSkill>) {
	entry.tree.remove(&entry.key);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::combo_node::ComboNode, items::slot_key::SlotKey, skills::Skill};
	use bevy::prelude::default;
	use common::components::Side;
	use std::collections::HashMap;

	#[test]
	fn insert_skill() {
		let mut tree = HashMap::from([]);
		let mut entry = NodeEntry {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.insert(Some(Skill {
			name: "my skill".to_owned(),
			..default()
		}));

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "my skill".to_owned(),
						..default()
					},
					default()
				)
			)]),
			tree
		);
	}

	#[test]
	fn insert_skill_without_changing_sub_tree() {
		let mut tree = HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill::default(),
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "sub tree skill".to_owned(),
							..default()
						},
						default(),
					),
				)]),
			),
		)]);
		let mut entry = NodeEntry {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.insert(Some(Skill {
			name: "my skill".to_owned(),
			..default()
		}));

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "my skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Main),
						(
							Skill {
								name: "sub tree skill".to_owned(),
								..default()
							},
							default(),
						),
					)]),
				),
			)]),
			tree
		);
	}

	#[test]
	fn insert_none_clears_corresponding_tree_entry() {
		let mut tree = HashMap::from([
			(
				SlotKey::Hand(Side::Main),
				(
					Skill::default(),
					ComboNode::new([(
						SlotKey::Hand(Side::Main),
						(
							Skill {
								name: "sub tree skill".to_owned(),
								..default()
							},
							default(),
						),
					)]),
				),
			),
			(SlotKey::Hand(Side::Off), (Skill::default(), default())),
		]);
		let mut entry = NodeEntry {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.insert(None);

		assert_eq!(
			HashMap::from([(SlotKey::Hand(Side::Off), (Skill::default(), default()))]),
			tree
		);
	}
}
