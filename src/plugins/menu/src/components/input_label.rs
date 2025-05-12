use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct InputLabel<TKey> {
	pub key: TKey,
}
