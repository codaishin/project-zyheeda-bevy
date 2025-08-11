use std::{collections::HashSet, hash::Hash};

use crate::{AppendSkill, Dropdown, KeySelect, traits::GetComponent};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{iteration::IterFinite, thread_safe::ThreadSafe},
};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct KeySelectDropdownCommand<TExtra, TKey = PlayerSlot> {
	pub(crate) extra: TExtra,
	pub(crate) key_path: Vec<TKey>,
}

impl<TKey> GetComponent for KeySelectDropdownCommand<AppendSkillCommand, TKey>
where
	TKey: ThreadSafe + IterFinite + PartialEq + Eq + Hash,
{
	type TComponent = Dropdown<KeySelect<AppendSkill<TKey>, TKey>>;
	type TInput = ExcludeKeys<TKey>;

	fn component(&self, ExcludeKeys(excluded): Self::TInput) -> Option<Self::TComponent> {
		let items = TKey::iterator()
			.filter(|key| !excluded.contains(key))
			.map(|on| KeySelect {
				extra: AppendSkill { on },
				key_path: self.key_path.clone(),
			})
			.collect::<Vec<_>>();

		if items.is_empty() {
			return None;
		}

		Some(Dropdown { items })
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct AppendSkillCommand;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ExcludeKeys<TKey>(pub HashSet<TKey>)
where
	TKey: Eq + Hash;

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::iteration::Iter;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Key {
		A,
		B,
		C,
	}

	impl IterFinite for _Key {
		fn iterator() -> Iter<Self> {
			Iter(Some(_Key::A))
		}

		fn next(Iter(current): &Iter<Self>) -> Option<Self> {
			let current = *current;
			match current? {
				_Key::A => Some(_Key::B),
				_Key::B => Some(_Key::C),
				_Key::C => None,
			}
		}
	}

	#[test]
	fn get_no_dropdown_when_all_keys_excluded() {
		let keys = _Key::iterator().collect();
		let command = KeySelectDropdownCommand {
			extra: AppendSkillCommand,
			key_path: vec![_Key::A],
		};

		assert_eq!(None, command.component(ExcludeKeys(keys)));
	}

	#[test]
	fn get_dropdown_dropdown_with_remaining_keys() {
		let keys = HashSet::from([_Key::B]);
		let command = KeySelectDropdownCommand {
			extra: AppendSkillCommand,
			key_path: vec![_Key::A],
		};

		assert_eq!(
			Some(Dropdown {
				items: vec![
					KeySelect {
						extra: AppendSkill { on: _Key::A },
						key_path: vec![_Key::A]
					},
					KeySelect {
						extra: AppendSkill { on: _Key::C },
						key_path: vec![_Key::A]
					}
				]
			}),
			command.component(ExcludeKeys(keys))
		);
	}
}
