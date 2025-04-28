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
	TKey: Copy,
	Key: From<TKey> + Hash + Eq + Copy,
	KeyCode: From<TKey> + Hash + Eq + Copy,
{
	fn update_key(&mut self, key: TKey, key_code: KeyCode) {
		self.0.update_key(key, key_code);
	}
}

#[derive(Debug, PartialEq)]
struct KeyMapInternal<TAllKeys = Key, TKeyCode = KeyCode>
where
	TAllKeys: Hash + Eq,
	TKeyCode: Hash + Eq,
{
	phantom_data: PhantomData<(TAllKeys, TKeyCode)>,
	key_to_key_code: HashMap<TAllKeys, TKeyCode>,
	key_code_to_key: HashMap<TKeyCode, TAllKeys>,
}

impl<TAllKeys, TKeyCode> Default for KeyMapInternal<TAllKeys, TKeyCode>
where
	TAllKeys: IterFinite + Copy + Hash + Eq,
	TKeyCode: Copy + Hash + Eq,
	TKeyCode: From<TAllKeys>,
{
	fn default() -> Self {
		let mut map = Self {
			phantom_data: PhantomData,
			key_to_key_code: HashMap::default(),
			key_code_to_key: HashMap::default(),
		};

		for key in TAllKeys::iterator() {
			map.update_key(key, TKeyCode::from(key));
		}

		map
	}
}

impl<TAllKeys, TKey, TKeyCode> GetKeyCode<TKey, TKeyCode> for KeyMapInternal<TAllKeys, TKeyCode>
where
	TKey: Copy,
	TAllKeys: From<TKey> + Hash + Eq,
	TKeyCode: From<TKey> + Copy + Hash + Eq,
{
	fn get_key_code(&self, value: TKey) -> TKeyCode {
		let Some(key_code) = self.key_to_key_code.get(&TAllKeys::from(value)) else {
			return TKeyCode::from(value);
		};

		*key_code
	}
}

impl<TAllKeys, TKey, TKeyCode> TryGetKey<TKeyCode, TKey> for KeyMapInternal<TAllKeys, TKeyCode>
where
	TAllKeys: Copy + Hash + Eq,
	TKey: TryFrom<TAllKeys>,
	TKeyCode: PartialEq + Hash + Eq,
{
	fn try_get_key(&self, key_code: TKeyCode) -> Option<TKey> {
		let key = self.key_code_to_key.get(&key_code)?;
		TKey::try_from(*key).ok()
	}
}

impl<TAllKeys, TKey, TKeyCode> UpdateKey<TKey, TKeyCode> for KeyMapInternal<TAllKeys, TKeyCode>
where
	TKey: Copy,
	TAllKeys: From<TKey> + Hash + Eq + Copy,
	TKeyCode: From<TKey> + Hash + Eq + Copy,
{
	fn update_key(&mut self, key: TKey, key_code: TKeyCode) {
		let old_key_code = self.get_key_code(key);
		let key = TAllKeys::from(key);

		match self.key_code_to_key.get(&key_code).copied() {
			Some(old_key) => {
				self.key_to_key_code.insert(old_key, old_key_code);
				self.key_code_to_key.insert(old_key_code, old_key);
			}
			None => {
				self.key_code_to_key.remove(&old_key_code);
			}
		}

		self.key_to_key_code.insert(key, key_code);
		self.key_code_to_key.insert(key_code, key);
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

	impl From<_AllKeys> for _Input {
		fn from(value: _AllKeys) -> Self {
			match value {
				_AllKeys::A(key) => _Input::from(key),
				_AllKeys::B(key) => _Input::from(key),
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

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Input {
		A,
		B,
		C,
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
	fn update_key() {
		let key = _KeyA;
		let key_code = _Input::B;
		let mut mapper = KeyMapInternal::<_AllKeys, _Input>::default();
		mapper.update_key(key, key_code);

		assert_eq!(
			(key_code, Some(key)),
			(mapper.get_key_code(key), mapper.try_get_key(key_code))
		);
	}

	#[test]
	fn update_key_removing_old_key_code_pairing() {
		let key = _KeyA;
		let key_code_b = _Input::B;
		let key_code_c = _Input::C;
		let mut mapper = KeyMapInternal::<_AllKeys, _Input>::default();
		mapper.update_key(key, key_code_b);
		mapper.update_key(key, key_code_c);

		assert_eq!(
			(key_code_c, Some(key), None as Option<_Input>),
			(
				mapper.get_key_code(key),
				mapper.try_get_key(key_code_c),
				mapper.try_get_key(key_code_b)
			)
		);
	}

	#[test]
	fn update_key_swapping_old_key() {
		let key_a = _KeyA;
		let key_b = _KeyB;
		let key_code_a = _Input::A;
		let key_code_b = _Input::B;
		let mut mapper = KeyMapInternal::<_AllKeys, _Input>::default();
		mapper.update_key(key_a, key_code_a);
		mapper.update_key(key_b, key_code_a);

		assert_eq!(
			(key_code_b, Some(key_b), key_code_a, Some(key_a),),
			(
				mapper.get_key_code(key_a),
				mapper.try_get_key(key_code_a),
				mapper.get_key_code(key_b),
				mapper.try_get_key(key_code_b),
			)
		);
	}
}
