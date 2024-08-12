use crate::{
	components::{
		dropdown::Dropdown,
		key_select::{AppendSkill, KeySelect},
		AppendSkillCommand,
		KeySelectDropdownInsertCommand,
	},
	traits::GetBundle,
};
use common::traits::iteration::IterFinite;
use skills::traits::{FollowupKeys, GetNode, RootKeys};

type Source<'a, TKey, TCombos> = (
	&'a KeySelectDropdownInsertCommand<AppendSkillCommand, TKey>,
	&'a TCombos,
);

impl<'a, TKey, TCombos> GetBundle for Source<'a, TKey, TCombos>
where
	TKey: IterFinite + PartialEq + Send + Sync + 'static,
	TCombos: GetNode<'a, Vec<TKey>, TNode: FollowupKeys<TItem = TKey>> + RootKeys<TItem = TKey>,
{
	type TBundle = Dropdown<KeySelect<AppendSkill<TKey>, TKey>>;

	fn bundle(&self) -> Option<Self::TBundle> {
		let (insert_command, combos) = self;

		let followups = match insert_command.key_path.is_empty() {
			true => combos.root_keys().collect::<Vec<_>>(),
			false => combos
				.node(&insert_command.key_path)
				.map(|e| e.followup_keys().collect::<Vec<_>>())
				.unwrap_or_default(),
		};

		let items: Vec<_> = TKey::iterator()
			.filter(|key| !followups.contains(key))
			.map(|key| KeySelect {
				extra: AppendSkill { on: key },
				key_path: insert_command.key_path.clone(),
			})
			.collect();

		if items.is_empty() {
			return None;
		}

		Some(Dropdown { items })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{dropdown::Dropdown, key_select::KeySelect};
	use common::traits::iteration::Iter;

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _Key {
		A,
		B,
		C,
	}

	impl IterFinite for _Key {
		fn iterator() -> Iter<Self> {
			Iter(Some(_Key::A))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_Key::A => Some(_Key::B),
				_Key::B => Some(_Key::C),
				_Key::C => None,
			}
		}
	}

	struct _Entry {
		follow_up_keys: Vec<_Key>,
	}

	impl FollowupKeys for _Entry {
		type TItem = _Key;

		fn followup_keys(&self) -> impl Iterator<Item = Self::TItem> {
			self.follow_up_keys.iter().cloned()
		}
	}

	macro_rules! impl_no_entry {
		($ident:ident) => {
			impl<'a> GetNode<'a, Vec<_Key>> for $ident {
				type TNode = _Entry;

				fn node(&'a self, _: &Vec<_Key>) -> Option<Self::TNode> {
					Some(_Entry {
						follow_up_keys: vec![],
					})
				}
			}
		};
	}

	macro_rules! impl_empty_root_keys {
		($ident:ident) => {
			impl RootKeys for $ident {
				type TItem = _Key;

				fn root_keys(&self) -> impl Iterator<Item = Self::TItem> {
					vec![].into_iter()
				}
			}
		};
	}

	#[test]
	fn insert_dropdown() {
		struct _Combos;

		impl_empty_root_keys!(_Combos);

		impl<'a> GetNode<'a, Vec<_Key>> for _Combos {
			type TNode = _Entry;

			fn node(&'a self, _: &Vec<_Key>) -> Option<Self::TNode> {
				Some(_Entry {
					follow_up_keys: vec![],
				})
			}
		}

		let source = (
			&KeySelectDropdownInsertCommand {
				extra: AppendSkillCommand,
				key_path: vec![_Key::A, _Key::B, _Key::C],
			},
			&_Combos,
		);

		assert_eq!(
			Some(Dropdown {
				items: vec![
					KeySelect {
						extra: AppendSkill { on: _Key::A },
						key_path: vec![_Key::A, _Key::B, _Key::C]
					},
					KeySelect {
						extra: AppendSkill { on: _Key::B },
						key_path: vec![_Key::A, _Key::B, _Key::C]
					},
					KeySelect {
						extra: AppendSkill { on: _Key::C },
						key_path: vec![_Key::A, _Key::B, _Key::C]
					},
				]
			}),
			source.bundle()
		)
	}

	#[test]
	fn get_dropdown_excluding_items_matching_any_followup_key() {
		struct _Combos;

		impl_empty_root_keys!(_Combos);

		impl<'a> GetNode<'a, Vec<_Key>> for _Combos {
			type TNode = _Entry;

			fn node(&'a self, _: &Vec<_Key>) -> Option<Self::TNode> {
				Some(_Entry {
					follow_up_keys: vec![_Key::A],
				})
			}
		}

		let source = (
			&KeySelectDropdownInsertCommand {
				extra: AppendSkillCommand,
				key_path: vec![_Key::A, _Key::B, _Key::C],
			},
			&_Combos,
		);

		assert_eq!(
			Some(Dropdown {
				items: vec![
					KeySelect {
						extra: AppendSkill { on: _Key::B },
						key_path: vec![_Key::A, _Key::B, _Key::C]
					},
					KeySelect {
						extra: AppendSkill { on: _Key::C },
						key_path: vec![_Key::A, _Key::B, _Key::C]
					},
				]
			}),
			source.bundle()
		)
	}

	#[test]
	fn retrieve_entry_with_correct_key_path() {
		struct _Combos;

		impl_empty_root_keys!(_Combos);

		static mut KEY_PATHS: Vec<Vec<_Key>> = vec![];

		impl<'a> GetNode<'a, Vec<_Key>> for _Combos {
			type TNode = _Entry;

			fn node(&'a self, key_path: &Vec<_Key>) -> Option<Self::TNode> {
				unsafe { KEY_PATHS.push(key_path.clone()) }
				None
			}
		}

		let key_path = vec![_Key::A, _Key::B, _Key::C];
		let source = (
			&KeySelectDropdownInsertCommand {
				extra: AppendSkillCommand,
				key_path: vec![_Key::A, _Key::B, _Key::C],
			},
			&_Combos,
		);

		_ = source.bundle();

		assert_eq!(vec![key_path], unsafe { KEY_PATHS.clone() },)
	}

	#[test]
	fn get_no_dropdown_when_follow_up_keys_already_contain_all_possible_keys() {
		struct _Combos;

		impl_empty_root_keys!(_Combos);

		impl<'a> GetNode<'a, Vec<_Key>> for _Combos {
			type TNode = _Entry;

			fn node(&'a self, _: &Vec<_Key>) -> Option<Self::TNode> {
				Some(_Entry {
					follow_up_keys: _Key::iterator().collect(),
				})
			}
		}

		let source = (
			&KeySelectDropdownInsertCommand {
				extra: AppendSkillCommand,
				key_path: vec![_Key::A, _Key::B, _Key::C],
			},
			&_Combos,
		);

		assert_eq!(None, source.bundle())
	}

	#[test]
	fn filter_against_root_keys_when_key_path_empty() {
		struct _Combos;

		impl_no_entry!(_Combos);

		impl RootKeys for _Combos {
			type TItem = _Key;

			fn root_keys(&self) -> impl Iterator<Item = Self::TItem> {
				vec![_Key::A, _Key::C].into_iter()
			}
		}

		let source = (
			&KeySelectDropdownInsertCommand {
				extra: AppendSkillCommand,
				key_path: vec![],
			},
			&_Combos,
		);

		assert_eq!(
			Some(Dropdown {
				items: vec![KeySelect {
					extra: AppendSkill { on: _Key::B },
					key_path: vec![]
				},]
			}),
			source.bundle()
		)
	}
}
