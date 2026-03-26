use bevy::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PluginSystemSet<T>(pub(crate) T)
where
	T: SystemSet;

impl<T> PluginSystemSet<T>
where
	T: SystemSet,
{
	pub const fn from_set(set: T) -> Self {
		Self(set)
	}
}
