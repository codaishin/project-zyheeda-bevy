use super::{ComboNode, NodeEntryMut};
use crate::traits::{Insert, ReKey};
use bevy::prelude::default;
use common::tools::{
	action_key::slot::SlotKey,
	ordered_hash_map::{Entry, OrderedHashMap},
};

impl<TSkill> Insert<Option<TSkill>> for NodeEntryMut<'_, TSkill> {
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

impl<TSkill> ReKey<SlotKey> for NodeEntryMut<'_, TSkill> {
	fn re_key(&mut self, new_key: SlotKey) {
		if self.key == new_key {
			return;
		}

		move_combo(self.tree, self.key, new_key);

		self.key = new_key;
	}
}

fn move_combo<TSkill>(
	tree: &mut OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
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
	src: &mut OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
	dst: &mut OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
) {
	let mut src = src
		.remove(dst_key)
		.map(|(_, ComboNode(src))| src)
		.unwrap_or_default();
	move_and_merge_branches_with_same_key(&mut src, dst);
	dst.extend(src);
}

fn move_and_merge_branches_with_same_key<TSkill>(
	src: &mut OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
	dst: &mut OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
) {
	for (key, (_, dst)) in dst.iter_mut() {
		move_and_merge_branch(key, src, &mut dst.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::combo_node::ComboNode, skills::Skill};
	use bevy::prelude::default;
	use common::{tools::action_key::slot::Side, traits::handles_localization::Token};

	#[test]
	fn insert_skill() {
		let mut tree = OrderedHashMap::from([]);
		let mut entry = NodeEntryMut {
			key: SlotKey::BottomHand(Side::Right),
			tree: &mut tree,
		};

		entry.insert(Some(Skill {
			token: Token::from("my skill"),
			..default()
		}));

		assert_eq!(
			OrderedHashMap::from([(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						token: Token::from("my skill"),
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
		let mut tree = OrderedHashMap::from([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill::default(),
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("sub tree skill"),
							..default()
						},
						default(),
					),
				)]),
			),
		)]);
		let mut entry = NodeEntryMut {
			key: SlotKey::BottomHand(Side::Right),
			tree: &mut tree,
		};

		entry.insert(Some(Skill {
			token: Token::from("my skill"),
			..default()
		}));

		assert_eq!(
			OrderedHashMap::from([(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						token: Token::from("my skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::BottomHand(Side::Right),
						(
							Skill {
								token: Token::from("sub tree skill"),
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
		let mut tree = OrderedHashMap::from([
			(
				SlotKey::BottomHand(Side::Right),
				(
					Skill::default(),
					ComboNode::new([(
						SlotKey::BottomHand(Side::Right),
						(
							Skill {
								token: Token::from("sub tree skill"),
								..default()
							},
							default(),
						),
					)]),
				),
			),
			(
				SlotKey::BottomHand(Side::Left),
				(Skill::default(), default()),
			),
		]);
		let mut entry = NodeEntryMut {
			key: SlotKey::BottomHand(Side::Right),
			tree: &mut tree,
		};

		entry.insert(None);

		assert_eq!(
			OrderedHashMap::from([(
				SlotKey::BottomHand(Side::Left),
				(Skill::default(), default())
			)]),
			tree
		);
	}

	#[test]
	fn rekey_sets_self_key_to_new_key() {
		let mut tree = OrderedHashMap::from([]);
		let mut entry = NodeEntryMut::<Skill> {
			key: SlotKey::BottomHand(Side::Right),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::BottomHand(Side::Left));

		assert_eq!(SlotKey::BottomHand(Side::Left), entry.key);
	}

	#[test]
	fn rekey_skill_to_other_key() {
		let mut tree = OrderedHashMap::from([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("my skill"),
					..default()
				},
				default(),
			),
		)]);
		let mut entry = NodeEntryMut {
			key: SlotKey::BottomHand(Side::Right),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::BottomHand(Side::Left));

		assert_eq!(
			OrderedHashMap::from([(
				SlotKey::BottomHand(Side::Left),
				(
					Skill {
						token: Token::from("my skill"),
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
		let mut tree = OrderedHashMap::from([
			(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						token: Token::from("my main -> off skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::BottomHand(Side::Left),
						(
							Skill {
								token: Token::from("my child off skill"),
								..default()
							},
							default(),
						),
					)]),
				),
			),
			(
				SlotKey::BottomHand(Side::Left),
				(
					Skill {
						token: Token::from("my off skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::BottomHand(Side::Right),
						(
							Skill {
								token: Token::from("my child main skill"),
								..default()
							},
							default(),
						),
					)]),
				),
			),
		]);
		let mut entry = NodeEntryMut {
			key: SlotKey::BottomHand(Side::Right),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::BottomHand(Side::Left));

		assert_eq!(
			OrderedHashMap::from([(
				SlotKey::BottomHand(Side::Left),
				(
					Skill {
						token: Token::from("my main -> off skill"),
						..default()
					},
					ComboNode::new([
						(
							SlotKey::BottomHand(Side::Left),
							(
								Skill {
									token: Token::from("my child off skill"),
									..default()
								},
								default(),
							),
						),
						(
							SlotKey::BottomHand(Side::Right),
							(
								Skill {
									token: Token::from("my child main skill"),
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
		let mut tree = OrderedHashMap::from([
			(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						token: Token::from("my main -> off skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::BottomHand(Side::Left),
						(
							Skill {
								token: Token::from("my child off skill"),
								..default()
							},
							ComboNode::new([(
								SlotKey::BottomHand(Side::Left),
								(
									Skill {
										token: Token::from("my child child off skill"),
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
				SlotKey::BottomHand(Side::Left),
				(
					Skill {
						token: Token::from("my off skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::BottomHand(Side::Left),
						(
							Skill {
								token: Token::from("my child off skill"),
								..default()
							},
							ComboNode::new([(
								SlotKey::BottomHand(Side::Right),
								(
									Skill {
										token: Token::from("my child child main skill"),
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
			key: SlotKey::BottomHand(Side::Right),
			tree: &mut tree,
		};

		entry.re_key(SlotKey::BottomHand(Side::Left));

		assert_eq!(
			OrderedHashMap::from([(
				SlotKey::BottomHand(Side::Left),
				(
					Skill {
						token: Token::from("my main -> off skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::BottomHand(Side::Left),
						(
							Skill {
								token: Token::from("my child off skill"),
								..default()
							},
							ComboNode::new([
								(
									SlotKey::BottomHand(Side::Left),
									(
										Skill {
											token: Token::from("my child child off skill"),
											..default()
										},
										default(),
									),
								),
								(
									SlotKey::BottomHand(Side::Right),
									(
										Skill {
											token: Token::from("my child child main skill"),
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
