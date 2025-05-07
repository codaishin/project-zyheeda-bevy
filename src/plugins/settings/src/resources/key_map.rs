pub(crate) mod dto;

use bevy::prelude::*;
use common::{
	tools::keys::{Key, user_input::UserInput},
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_settings::UpdateKey,
		iterate::Iterate,
		iteration::IterFinite,
		key_mappings::{GetUserInput, TryGetKey},
		load_asset::LoadAsset,
	},
};
use dto::KeyMapDto;
use std::{
	collections::{
		HashMap,
		HashSet,
		hash_map::{Entry, Iter},
	},
	error::Error,
	fmt::{Debug, Display},
	hash::Hash,
	marker::PhantomData,
};

#[derive(Resource, Asset, TypePath, Default, Debug, PartialEq, Clone)]
pub struct KeyMap(KeyMapInternal);

impl<TKey> GetUserInput<TKey, UserInput> for KeyMap
where
	TKey: Copy,
	Key: From<TKey>,
	UserInput: From<TKey>,
{
	fn get_key_code(&self, key: TKey) -> UserInput {
		self.0.get_key_code(key)
	}
}

impl<TKey> TryGetKey<UserInput, TKey> for KeyMap
where
	TKey: TryFrom<Key> + Copy,
	UserInput: From<TKey> + PartialEq,
{
	fn try_get_key(&self, key: UserInput) -> Option<TKey> {
		self.0.try_get_key(key)
	}
}

impl<TKey> UpdateKey<TKey, UserInput> for KeyMap
where
	TKey: Copy,
	Key: From<TKey> + Hash + Eq + Copy,
	UserInput: From<TKey> + Hash + Eq + Copy,
{
	fn update_key(&mut self, key: TKey, key_code: UserInput) {
		self.0.update_key(key, key_code);
	}
}

impl TryLoadFrom<KeyMapDto<Key, UserInput>> for KeyMap {
	type TInstantiationError = RepeatedAssignments<Key, UserInput>;

	fn try_load_from<TLoadAsset>(
		dto: KeyMapDto<Key, UserInput>,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		KeyMapInternal::try_load_from(dto, asset_server).map(KeyMap)
	}
}

#[derive(Debug, PartialEq, Clone)]
struct KeyMapInternal<TAllKeys = Key, TKeyCode = UserInput>
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

impl<TAllKeys, TKey, TKeyCode> GetUserInput<TKey, TKeyCode> for KeyMapInternal<TAllKeys, TKeyCode>
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

impl<TAllKeys, TKeyCode> TryLoadFrom<KeyMapDto<TAllKeys, TKeyCode>>
	for KeyMapInternal<TAllKeys, TKeyCode>
where
	TAllKeys: IterFinite + Debug + Copy + Hash + Eq + TypePath,
	TKeyCode: Debug + Copy + Hash + Eq + TypePath,
	TKeyCode: From<TAllKeys>,
{
	type TInstantiationError = RepeatedAssignments<TAllKeys, TKeyCode>;

	fn try_load_from<TLoadAsset>(
		KeyMapDto { keys }: KeyMapDto<TAllKeys, TKeyCode>,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		let mut mapper = Self::default();
		for (key, key_code) in keys {
			mapper.update_key(key, key_code);
		}

		if let Some(repeated) = RepeatedAssignments::from_mapper(&mapper) {
			return Err(repeated);
		}

		Ok(mapper)
	}
}

#[derive(TypePath, Debug, PartialEq)]
pub struct RepeatedAssignments<TAllKeys, TKeyCode>(HashMap<TKeyCode, HashSet<TAllKeys>>)
where
	TAllKeys: Debug + Eq + Hash + TypePath,
	TKeyCode: Debug + Eq + Hash + TypePath;

impl<TAllKeys, TKeyCode> RepeatedAssignments<TAllKeys, TKeyCode>
where
	TAllKeys: Debug + Eq + Hash + TypePath + Copy,
	TKeyCode: Debug + Eq + Hash + TypePath + Copy,
{
	fn from_mapper(mapper: &KeyMapInternal<TAllKeys, TKeyCode>) -> Option<Self> {
		let mut repeated = Self(HashMap::default());

		for (key, input) in &mapper.key_to_key_code {
			match repeated.0.entry(*input) {
				Entry::Occupied(mut entry) => {
					entry.get_mut().insert(*key);
				}
				Entry::Vacant(entry) => {
					entry.insert(HashSet::from([*key]));
				}
			}
		}

		repeated.0.retain(|_, keys| keys.len() > 1);

		if repeated.0.is_empty() {
			return None;
		}

		Some(repeated)
	}
}

impl<TAllKeys, TKeyCode> Display for RepeatedAssignments<TAllKeys, TKeyCode>
where
	TAllKeys: Debug + Eq + Hash + TypePath,
	TKeyCode: Debug + Eq + Hash + TypePath,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Keys have been assigned multiple times: {:#?}", self)
	}
}

impl<TAllKeys, TKeyCode> Error for RepeatedAssignments<TAllKeys, TKeyCode>
where
	TAllKeys: Debug + Eq + Hash + TypePath,
	TKeyCode: Debug + Eq + Hash + TypePath,
{
}

impl<'a> Iterate<'a> for KeyMap {
	type TItem = (&'a Key, &'a UserInput);
	type TIter = Iter<'a, Key, UserInput>;

	fn iterate(&'a self) -> Self::TIter {
		self.0.key_to_key_code.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::iteration::{Iter, IterFinite};

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Input {
		A,
		B,
		C,
	}

	mod map {
		use super::*;

		#[test]
		fn to_input() {
			let mapper = KeyMapInternal::<_AllKeys, _Input>::default();
			let mapped = mapper.get_key_code(_KeyB);

			assert_eq!(_Input::B, mapped,);
		}

		#[test]
		fn to_key_a() {
			let mapper = KeyMapInternal::<_AllKeys, _Input>::default();
			let mapped = mapper.try_get_key(_Input::A);

			assert_eq!(Some(_KeyA), mapped);
		}

		#[test]
		fn to_key_b() {
			let mapper = KeyMapInternal::<_AllKeys, _Input>::default();
			let mapped = mapper.try_get_key(_Input::B);

			assert_eq!(Some(_KeyB), mapped);
		}
	}

	mod update {
		use super::*;

		#[test]
		fn key() {
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
		fn key_removing_old_key_code_pairing() {
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
		fn key_swapping_old_key() {
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

	mod try_load {
		use super::*;

		struct _Server;

		impl LoadAsset for _Server {
			fn load_asset<TAsset, TPath>(&mut self, _: TPath) -> Handle<TAsset>
			where
				TAsset: Asset,
			{
				panic!("NUT USED")
			}
		}

		#[test]
		fn from_dto() -> Result<(), RepeatedAssignments<_AllKeys, _Input>> {
			let dto = KeyMapDto::from([(_AllKeys::A(_KeyA), _Input::C)]);

			let mapper = KeyMapInternal::try_load_from(dto, &mut _Server)?;

			assert_eq!(
				(_Input::C, Some(_KeyA), _Input::B, Some(_KeyB)),
				(
					mapper.get_key_code(_KeyA),
					mapper.try_get_key(_Input::C),
					mapper.get_key_code(_KeyB),
					mapper.try_get_key(_Input::B),
				)
			);
			Ok(())
		}

		#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
		enum _FaultyKey {
			A,
			B,
			C,
		}

		impl IterFinite for _FaultyKey {
			fn iterator() -> Iter<Self> {
				Iter(Some(_FaultyKey::A))
			}

			fn next(current: &Iter<Self>) -> Option<Self> {
				match current.0? {
					_FaultyKey::A => Some(_FaultyKey::B),
					_FaultyKey::B => Some(_FaultyKey::C),
					_FaultyKey::C => None,
				}
			}
		}

		impl From<_FaultyKey> for _Input {
			fn from(value: _FaultyKey) -> Self {
				match value {
					_FaultyKey::A => _Input::A,
					_FaultyKey::B => _Input::C, // this is the faulty mapping
					_FaultyKey::C => _Input::C,
				}
			}
		}

		#[test]
		fn from_dto_error_when_input_assigned_twice() {
			let mapper = KeyMapInternal::try_load_from(KeyMapDto::from([]), &mut _Server);

			assert_eq!(
				Err(RepeatedAssignments(HashMap::from([(
					_Input::C,
					HashSet::from([_FaultyKey::B, _FaultyKey::C])
				)]))),
				mapper
			);
		}
	}
}
