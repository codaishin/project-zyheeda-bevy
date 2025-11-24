use super::{GetHandle, WrapHandle};
use bevy::prelude::*;

impl WrapHandle for AnimationGraph {
	type TComponent = AnimationGraphHandle;

	fn wrap_handle(handle: Handle<Self>) -> Self::TComponent {
		AnimationGraphHandle(handle)
	}
}

impl GetHandle for AnimationGraphHandle {
	type TAsset = AnimationGraph;

	fn get_handle(&self) -> &Handle<Self::TAsset> {
		&self.0
	}
}
