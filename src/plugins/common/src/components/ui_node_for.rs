use crate::traits::handles_graphics::StaticRenderLayers;
use bevy::{prelude::*, render::view::RenderLayers};
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
pub struct UiNodeFor<T> {
	pub owner: Entity,
	owner_type: PhantomData<T>,
}

impl<T> UiNodeFor<T> {
	pub fn with(owner: Entity) -> Self {
		Self {
			owner,
			owner_type: PhantomData,
		}
	}

	pub fn render_layer<TUiCamera>() -> RenderLayers
	where
		TUiCamera: StaticRenderLayers,
	{
		TUiCamera::render_layers()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ui_node_set_render_layer() {
		struct _T;

		impl StaticRenderLayers for _T {
			fn render_layers() -> RenderLayers {
				RenderLayers::layer(42)
			}
		}

		assert_eq!(
			RenderLayers::layer(42),
			UiNodeFor::<()>::render_layer::<_T>()
		);
	}
}
