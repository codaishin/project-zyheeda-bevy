use bevy::prelude::*;

pub trait AssetMarker: internal::AssetMarker {}

impl<T> AssetMarker for T where T: internal::AssetMarker {}

impl internal::AssetMarker for Mesh {
	type TComponent = Mesh3d;

	fn wrap(handle: Handle<Self>) -> Self::TComponent {
		Mesh3d(handle)
	}
}

impl internal::AssetMarker for StandardMaterial {
	type TComponent = MeshMaterial3d<StandardMaterial>;

	fn wrap(handle: Handle<Self>) -> Self::TComponent {
		MeshMaterial3d(handle)
	}
}

pub(crate) mod internal {
	use super::*;

	pub trait AssetMarker: Asset + Sized {
		type TComponent: Component;

		fn wrap(handle: Handle<Self>) -> Self::TComponent;
	}
}
