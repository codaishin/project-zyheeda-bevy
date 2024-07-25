use super::NodeEntry;
use crate::{
	items::slot_key::SlotKey,
	traits::{Insert, ReKey},
};
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

impl<'a, TSkill> ReKey<SlotKey> for NodeEntry<'a, TSkill> {
	fn re_key(&mut self, other_key: SlotKey) {
		if self.key == other_key {
			return;
		}

		let self_node = self.tree.remove(&self.key);
		let other_node = self.tree.remove(&other_key);

		if let Some(self_node) = self_node {
			self.tree.insert(other_key, self_node);
		};

		if let Some(other_node) = other_node {
			self.tree.insert(self.key, other_node);
		}

		self.key = other_key;
	}
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

	#[test]
	fn rekey_skill_to_other_key() {
		let mut tree = HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				default(),
			),
		)]);
		let mut entry = NodeEntry {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::Hand(Side::Off));

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Off),
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
	fn rekey_sets_self_key_to_new_key() {
		let mut tree = HashMap::from([]);
		let mut entry = NodeEntry::<Skill> {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::Hand(Side::Off));

		assert_eq!(SlotKey::Hand(Side::Off), entry.key);
	}

	#[test]
	fn rekey_swaps_skills_if_target_key_is_used_in_tree() {
		let mut tree = HashMap::from([
			(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "my skill".to_owned(),
						..default()
					},
					default(),
				),
			),
			(
				SlotKey::Hand(Side::Off),
				(
					Skill {
						name: "my other skill".to_owned(),
						..default()
					},
					default(),
				),
			),
		]);
		let mut entry = NodeEntry {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::Hand(Side::Off));

		assert_eq!(
			HashMap::from([
				(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "my skill".to_owned(),
							..default()
						},
						default()
					)
				),
				(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "my other skill".to_owned(),
							..default()
						},
						default(),
					)
				),
			]),
			tree
		);
	}
}
