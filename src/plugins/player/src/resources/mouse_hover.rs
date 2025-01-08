use bevy::prelude::*;
use common::{tools::collider_info::ColliderInfo, traits::accessors::get::GetterRefOptional};

#[derive(Resource, Debug, PartialEq, Clone)]
pub struct MouseHover<T = Entity>(pub(crate) Option<ColliderInfo<T>>);

impl<T> Default for MouseHover<T> {
	fn default() -> Self {
		Self(None)
	}
}

impl<T> GetterRefOptional<ColliderInfo<T>> for MouseHover<T> {
	fn get(&self) -> Option<&ColliderInfo<T>> {
		self.0.as_ref()
	}
}
