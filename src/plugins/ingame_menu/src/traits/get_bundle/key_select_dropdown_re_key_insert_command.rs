use crate::{
	components::{
		dropdown::Dropdown,
		key_select::{KeySelect, ReKeySkill},
		KeySelectDropdownInsertCommand,
		ReKeyCommand,
	},
	traits::GetBundle,
};
use common::traits::iteration::IterFinite;

type Source<'a, TKey, TCombos> = (
	&'a KeySelectDropdownInsertCommand<ReKeyCommand<TKey>, TKey>,
	&'a TCombos,
);

impl<'a, TKey, TCombos> GetBundle for Source<'a, TKey, TCombos>
where
	TKey: IterFinite + PartialEq + Send + Sync + 'static,
{
	type TBundle = Dropdown<KeySelect<ReKeySkill<TKey>, TKey>>;

	fn bundle(&self) -> Self::TBundle {
		let (insert_command, ..) = self;
		let items = TKey::iterator()
			.filter(|key| key != &insert_command.extra.ignore)
			.map(|key| KeySelect {
				extra: ReKeySkill { to: key },
				key_path: insert_command.key_path.clone(),
			})
			.collect();

		Dropdown { items }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		dropdown::Dropdown,
		key_select::{KeySelect, ReKeySkill},
	};
	use bevy::prelude::Component;
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

	#[derive(Component)]
	struct _Combos;

	#[test]
	fn insert_dropdown() {
		let source = (
			&KeySelectDropdownInsertCommand {
				extra: ReKeyCommand { ignore: _Key::C },
				key_path: vec![_Key::A, _Key::B, _Key::C],
			},
			&_Combos,
		);

		assert_eq!(
			Dropdown {
				items: vec![
					KeySelect {
						extra: ReKeySkill { to: _Key::A },
						key_path: vec![_Key::A, _Key::B, _Key::C],
					},
					KeySelect {
						extra: ReKeySkill { to: _Key::B },
						key_path: vec![_Key::A, _Key::B, _Key::C],
					},
				]
			},
			source.bundle()
		)
	}
}
