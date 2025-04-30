use super::{
	iterate::Iterate,
	key_mappings::{GetKeyCode, TryGetKey},
};
use crate::tools::keys::Key;
use bevy::prelude::*;

pub trait HandlesSettings {
	type TKeyMap<TKey>: Resource
		+ GetKeyCode<TKey, KeyCode>
		+ TryGetKey<KeyCode, TKey>
		+ UpdateKey<TKey, KeyCode>
		+ for<'a> Iterate<'a, TItem = (&'a Key, &'a KeyCode)>
	where
		Key: From<TKey>,
		TKey: TryFrom<Key> + Copy,
		KeyCode: From<TKey> + PartialEq;
}

pub trait UpdateKey<TKey, TKeyCode> {
	fn update_key(&mut self, key: TKey, key_code: TKeyCode);
}
