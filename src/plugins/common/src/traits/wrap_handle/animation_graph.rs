use super::{UnwrapHandle, WrapHandle};
use bevy::prelude::*;

impl WrapHandle for AnimationGraph {
	type TComponent = AnimationGraphHandle;

	fn wrap(handle: Handle<Self>) -> Self::TComponent {
		AnimationGraphHandle(handle)
	}
}

impl UnwrapHandle for AnimationGraphHandle {
	type TAsset = AnimationGraph;

	fn unwrap(&self) -> &Handle<Self::TAsset> {
		&self.0
	}
}
