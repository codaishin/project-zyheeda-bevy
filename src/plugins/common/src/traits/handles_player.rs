use bevy::{math::InvalidDirectionError, prelude::*};

pub trait HandlesPlayer {
	type TPlayer: Component;
}

pub trait PlayerMainCamera {
	type TPlayerMainCamera: Component + Default;
}

#[derive(Debug, PartialEq)]
pub enum DirectionError<TKey> {
	Invalid(InvalidDirectionError),
	KeyHasNoDirection(TKey),
}
