use crate::{
	tools::keys::Key,
	traits::{
		iteration::IterFinite,
		key_mappings::{GetKeyCode, TryGetKey},
	},
};
use bevy::{ecs::system::Resource, input::keyboard::KeyCode};
use std::marker::PhantomData;

#[derive(Resource, Default, Debug, PartialEq)]
pub struct KeyMap(KeyMapInternal);

impl<TKey> GetKeyCode<TKey, KeyCode> for KeyMap
where
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

//FIXME: Add the possibility to override defaults
#[derive(Debug, PartialEq)]
struct KeyMapInternal<TAllKeys = Key, TKeyCode = KeyCode> {
	phantom_data: PhantomData<(TAllKeys, TKeyCode)>,
}

impl<TKey, TInput> Default for KeyMapInternal<TKey, TInput> {
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}
}

impl<TAllKeys, TKey, TKeyCode> GetKeyCode<TKey, TKeyCode> for KeyMapInternal<TAllKeys, TKeyCode>
where
	TKeyCode: From<TKey>,
{
	fn get_key_code(&self, value: TKey) -> TKeyCode {
		TKeyCode::from(value)
	}
}

impl<TAllKeys, TKey, TKeyCode> TryGetKey<TKeyCode, TKey> for KeyMapInternal<TAllKeys, TKeyCode>
where
	TAllKeys: IterFinite + Copy,
	TKey: TryFrom<TAllKeys> + Copy,
	TKeyCode: From<TKey> + PartialEq,
{
	fn try_get_key(&self, value: TKeyCode) -> Option<TKey> {
		TAllKeys::iterator()
			.filter_map(|key| TKey::try_from(key).ok())
			.find(|key| value == TKeyCode::from(*key))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::iteration::{Iter, IterFinite};

	#[derive(Debug, PartialEq, Clone, Copy)]
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

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _KeyA;

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

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _KeyB;

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

	#[derive(Debug, PartialEq)]
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
}
