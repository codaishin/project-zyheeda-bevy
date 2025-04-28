use super::key_mappings::{GetKeyCode, TryGetKey};
use crate::tools::keys::Key;
use bevy::prelude::*;

pub trait HandlesSettings {
	type TKeyMap<TKey>: Resource + GetKeyCode<TKey, KeyCode> + TryGetKey<KeyCode, TKey>
	where
		TKey: TryFrom<Key> + Copy,
		KeyCode: From<TKey> + PartialEq;
}
