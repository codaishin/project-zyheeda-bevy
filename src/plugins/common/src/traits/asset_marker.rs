use bevy::prelude::*;

pub trait AssetMarker: internal::AssetMarker {
	type TComponent: Component;

	fn component(handle: Handle<Self>) -> Self::TComponent;
}

impl<T> AssetMarker for T
where
	T: internal::AssetMarker,
{
	type TComponent = T::TWrapper;

	fn component(handle: Handle<Self>) -> Self::TComponent {
		Self::wrap(handle)
	}
}

impl internal::AssetMarker for Mesh {
	type TWrapper = Mesh3d;

	fn wrap(handle: Handle<Self>) -> Self::TWrapper {
		Mesh3d(handle)
	}
}

impl internal::AssetMarker for StandardMaterial {
	type TWrapper = MeshMaterial3d<StandardMaterial>;

	fn wrap(handle: Handle<Self>) -> Self::TWrapper {
		MeshMaterial3d(handle)
	}
}

pub(crate) mod internal {
	use super::*;

	pub trait AssetMarker: Asset + Sized {
		type TWrapper: Component;

		fn wrap(handle: Handle<Self>) -> Self::TWrapper;
	}
}
