use super::{ComboNode, NodeEntryMut};
use crate::{
	items::slot_key::SlotKey,
	traits::{Insert, ReKey},
};
use bevy::prelude::default;
use std::collections::{hash_map::Entry, HashMap};

impl<'a, TSkill> Insert<Option<TSkill>> for NodeEntryMut<'a, TSkill> {
	fn insert(&mut self, value: Option<TSkill>) {
		match value {
			Some(value) => update_entry(self, value),
			None => clear_entry(self),
		}
	}
}

fn update_entry<TSkill>(entry: &mut NodeEntryMut<TSkill>, value: TSkill) {
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

fn clear_entry<TSkill>(entry: &mut NodeEntryMut<TSkill>) {
	entry.tree.remove(&entry.key);
}

impl<'a, TSkill> ReKey<SlotKey> for NodeEntryMut<'a, TSkill> {
	fn re_key(&mut self, new_key: SlotKey) {
		if self.key == new_key {
			return;
		}

		move_combo(self.tree, self.key, new_key);

		self.key = new_key;
	}
}

fn move_combo<TSkill>(
	tree: &mut HashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
	src_key: SlotKey,
	dst_key: SlotKey,
) {
	let Some((skill, ComboNode(mut children))) = tree.remove(&src_key) else {
		return;
	};

	move_and_merge_branch(&dst_key, tree, &mut children);
	tree.insert(dst_key, (skill, ComboNode(children)));
}

fn move_and_merge_branch<TSkill>(
	dst_key: &SlotKey,
	src: &mut HashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
	dst: &mut HashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
) {
	let mut src = src
		.remove(dst_key)
		.map(|(_, ComboNode(src))| src)
		.unwrap_or_default();
	move_and_merge_branches_with_same_key(&mut src, dst);
	dst.extend(src);
}

fn move_and_merge_branches_with_same_key<TSkill>(
	src: &mut HashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
	dst: &mut HashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
) {
	for (key, (_, ComboNode(dst))) in dst.iter_mut() {
		move_and_merge_branch(key, src, dst)
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
		let mut entry = NodeEntryMut {
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
		let mut entry = NodeEntryMut {
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
		let mut entry = NodeEntryMut {
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
	fn rekey_sets_self_key_to_new_key() {
		let mut tree = HashMap::from([]);
		let mut entry = NodeEntryMut::<Skill> {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::Hand(Side::Off));

		assert_eq!(SlotKey::Hand(Side::Off), entry.key);
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
		let mut entry = NodeEntryMut {
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
	fn rekey_skill_merge_with_tree_on_other_key() {
		let mut tree = HashMap::from([
			(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "my main -> off skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "my child off skill".to_owned(),
								..default()
							},
							default(),
						),
					)]),
				),
			),
			(
				SlotKey::Hand(Side::Off),
				(
					Skill {
						name: "my off skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Main),
						(
							Skill {
								name: "my child main skill".to_owned(),
								..default()
							},
							default(),
						),
					)]),
				),
			),
		]);
		let mut entry = NodeEntryMut {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::Hand(Side::Off));

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Off),
				(
					Skill {
						name: "my main -> off skill".to_owned(),
						..default()
					},
					ComboNode::new([
						(
							SlotKey::Hand(Side::Off),
							(
								Skill {
									name: "my child off skill".to_owned(),
									..default()
								},
								default(),
							),
						),
						(
							SlotKey::Hand(Side::Main),
							(
								Skill {
									name: "my child main skill".to_owned(),
									..default()
								},
								default(),
							),
						)
					]),
				)
			)]),
			tree
		);
	}

	#[test]
	fn rekey_skill_merge_with_tree_on_other_key_deeply() {
		let mut tree = HashMap::from([
			(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "my main -> off skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "my child off skill".to_owned(),
								..default()
							},
							ComboNode::new([(
								SlotKey::Hand(Side::Off),
								(
									Skill {
										name: "my child child off skill".to_owned(),
										..default()
									},
									default(),
								),
							)]),
						),
					)]),
				),
			),
			(
				SlotKey::Hand(Side::Off),
				(
					Skill {
						name: "my off skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "my child off skill".to_owned(),
								..default()
							},
							ComboNode::new([(
								SlotKey::Hand(Side::Main),
								(
									Skill {
										name: "my child child main skill".to_owned(),
										..default()
									},
									default(),
								),
							)]),
						),
					)]),
				),
			),
		]);
		let mut entry = NodeEntryMut {
			key: SlotKey::Hand(Side::Main),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::Hand(Side::Off));

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Off),
				(
					Skill {
						name: "my main -> off skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "my child off skill".to_owned(),
								..default()
							},
							ComboNode::new([
								(
									SlotKey::Hand(Side::Off),
									(
										Skill {
											name: "my child child off skill".to_owned(),
											..default()
										},
										default(),
									),
								),
								(
									SlotKey::Hand(Side::Main),
									(
										Skill {
											name: "my child child main skill".to_owned(),
											..default()
										},
										default(),
									),
								)
							]),
						),
					)]),
				)
			)]),
			tree
		);
	}
}
