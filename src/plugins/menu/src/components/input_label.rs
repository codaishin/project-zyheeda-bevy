use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Node)]
pub struct InputLabel<TKey> {
	pub key: TKey,
}
