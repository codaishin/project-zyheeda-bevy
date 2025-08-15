use bevy::prelude::*;
use common::tools::collider_info::ColliderInfo;

#[derive(Resource, Debug, PartialEq, Clone)]
pub struct MouseHover<T = Entity>(pub(crate) Option<ColliderInfo<T>>);

impl<T> Default for MouseHover<T> {
	fn default() -> Self {
		Self(None)
	}
}

impl<'a, T> From<&'a MouseHover<T>> for Option<&'a ColliderInfo<T>> {
	fn from(MouseHover(info): &'a MouseHover<T>) -> Self {
		info.as_ref()
	}
}
