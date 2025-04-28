use bevy::{ecs::system::Resource, input::keyboard::KeyCode};
use common::{
	tools::keys::Key,
	traits::{
		handles_settings::UpdateKey,
		iteration::IterFinite,
		key_mappings::{GetKeyCode, TryGetKey},
	},
};
use std::{collections::HashMap, hash::Hash, marker::PhantomData};

#[derive(Resource, Default, Debug, PartialEq)]
pub struct KeyMap(KeyMapInternal);

impl<TKey> GetKeyCode<TKey, KeyCode> for KeyMap
where
	TKey: Copy,
	Key: From<TKey>,
	KeyCode: From<TKey>,
{
	fn get_key_code(&self, key: TKey) -> KeyCode {
		self.0.get_key_code(key)
	}
}

impl<TKey> TryGetKey<KeyCode, TKey> for KeyMap
where
	TKey: TryFrom<Key> + Copy,
	KeyCode: From<TKey> + PartialEq,
{
	fn try_get_key(&self, key: KeyCode) -> Option<TKey> {
		self.0.try_get_key(key)
	}
}

impl<TKey> UpdateKey<TKey, KeyCode> for KeyMap
where
	Key: From<TKey>,
{
	fn update_key(&mut self, key: TKey, key_code: KeyCode) {
		self.0.update_key(key, key_code);
	}
}

#[derive(Debug, PartialEq)]
struct KeyMapInternal<TAllKeys = Key, TKeyCode = KeyCode>
where
	TAllKeys: Hash + Eq,
{
	phantom_data: PhantomData<(TAllKeys, TKeyCode)>,
	key_overrides: HashMap<TAllKeys, TKeyCode>,
}

impl<TKey, TInput> Default for KeyMapInternal<TKey, TInput>
where
	TKey: Hash + Eq,
{
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
			key_overrides: HashMap::default(),
		}
	}
}

impl<TAllKeys, TKey, TKeyCode> GetKeyCode<TKey, TKeyCode> for KeyMapInternal<TAllKeys, TKeyCode>
where
	TKey: Copy,
	TAllKeys: From<TKey> + Hash + Eq,
	TKeyCode: From<TKey> + Copy,
{
	fn get_key_code(&self, value: TKey) -> TKeyCode {
		let Some(key_code) = self.key_overrides.get(&TAllKeys::from(value)) else {
			return TKeyCode::from(value);
		};

		*key_code
	}
}

impl<TAllKeys, TKey, TKeyCode> TryGetKey<TKeyCode, TKey> for KeyMapInternal<TAllKeys, TKeyCode>
where
	TAllKeys: IterFinite + Copy + Hash + Eq,
	TKey: TryFrom<TAllKeys> + Copy,
	TKeyCode: From<TKey> + PartialEq,
{
	fn try_get_key(&self, value: TKeyCode) -> Option<TKey> {
		let override_key = self
			.key_overrides
			.iter()
			.find_map(|(key, key_code)| match key_code {
				key_code if key_code == &value => TKey::try_from(*key).ok(),
				_ => None,
			});

		let Some(override_key) = override_key else {
			return TAllKeys::iterator()
				.filter_map(|key| TKey::try_from(key).ok())
				.find(|key| value == TKeyCode::from(*key));
		};

		Some(override_key)
	}
}

impl<TAllKeys, TKey, TKeyCode> UpdateKey<TKey, TKeyCode> for KeyMapInternal<TAllKeys, TKeyCode>
where
	TAllKeys: From<TKey> + Hash + Eq,
{
	fn update_key(&mut self, key: TKey, key_code: TKeyCode) {
		self.key_overrides.insert(TAllKeys::from(key), key_code);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::iteration::{Iter, IterFinite};

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _AllKeys {
		A(_KeyA),
		B(_KeyB),
	}

	impl IterFinite for _AllKeys {
		fn iterator() -> Iter<Self> {
			Iter(Some(_AllKeys::A(_KeyA)))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_AllKeys::A(_KeyA) => Some(_AllKeys::B(_KeyB)),
				_AllKeys::B(_KeyB) => None,
			}
		}
	}

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _KeyA;

	impl From<_KeyA> for _AllKeys {
		fn from(key: _KeyA) -> Self {
			Self::A(key)
		}
	}

	impl TryFrom<_AllKeys> for _KeyA {
		type Error = ();

		fn try_from(key: _AllKeys) -> Result<Self, Self::Error> {
			match key {
				_AllKeys::A(key) => Ok(key),
				_ => Err(()),
			}
		}
	}

	impl From<_KeyA> for _Input {
		fn from(_: _KeyA) -> Self {
			_Input::A
		}
	}

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _KeyB;

	impl From<_KeyB> for _AllKeys {
		fn from(key: _KeyB) -> Self {
			Self::B(key)
		}
	}

	impl TryFrom<_AllKeys> for _KeyB {
		type Error = ();

		fn try_from(key: _AllKeys) -> Result<Self, Self::Error> {
			match key {
				_AllKeys::B(key) => Ok(key),
				_ => Err(()),
			}
		}
	}

	impl From<_KeyB> for _Input {
		fn from(_: _KeyB) -> Self {
			_Input::B
		}
	}

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _Input {
		A,
		B,
	}

	#[test]
	fn map_to_input() {
		let mapper = KeyMapInternal::<_AllKeys, _Input>::default();
		let mapped = mapper.get_key_code(_KeyB);

		assert_eq!(_Input::B, mapped,);
	}

	#[test]
	fn map_to_key_a() {
		let mapper = KeyMapInternal::<_AllKeys, _Input>::default();
		let mapped = mapper.try_get_key(_Input::A);

		assert_eq!(Some(_KeyA), mapped);
	}

	#[test]
	fn map_to_key_b() {
		let mapper = KeyMapInternal::<_AllKeys, _Input>::default();
		let mapped = mapper.try_get_key(_Input::B);

		assert_eq!(Some(_KeyB), mapped);
	}

	#[test]
	fn update_slot_key() {
		let key = _KeyA;
		let key_code = _Input::B;
		let mut mapper = KeyMapInternal::<_AllKeys, _Input>::default();
		mapper.update_key(key, key_code);

		assert_eq!(
			(key_code, Some(key)),
			(mapper.get_key_code(key), mapper.try_get_key(key_code))
		);
	}
}
