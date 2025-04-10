mod animation_graph;

use bevy::prelude::*;

pub trait WrapHandle: Asset + Sized {
	type TComponent: UnwrapHandle<TAsset = Self>;

	fn wrap(handle: Handle<Self>) -> Self::TComponent;
}

pub trait UnwrapHandle: Component {
	type TAsset: Asset;

	fn unwrap(&self) -> &Handle<Self::TAsset>;
}
