mod animation_graph;

use bevy::prelude::*;

pub trait WrapHandle: Asset + Sized {
	type TComponent: GetHandle<TAsset = Self>;

	fn wrap_handle(handle: Handle<Self>) -> Self::TComponent;
}

pub trait GetHandle: Component {
	type TAsset: Asset;

	fn get_handle(&self) -> &Handle<Self::TAsset>;
}
