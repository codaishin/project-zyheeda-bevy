pub mod essence;
pub mod flip;

use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier3d::prelude::*;
use flip::FlipHorizontally;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::traits::handles_graphics::StaticRenderLayers;

#[derive(Debug, PartialEq, Clone)]
pub struct Swap<T1, T2>(pub T1, pub T2);

#[derive(Component, Debug, PartialEq)]
pub struct Collection<TElement>(pub Vec<TElement>);

impl<TElement> Collection<TElement> {
	pub fn new<const N: usize>(elements: [TElement; N]) -> Self {
		Self(elements.into())
	}
}

#[derive(Component)]
pub struct GroundOffset(pub Vec3);

#[derive(Component, Debug, PartialEq)]
pub struct Immobilized;

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
#[require(Collider, Transform, ActiveEvents, ActiveCollisionTypes)]
pub struct ColliderRoot(pub Entity);

#[derive(Component, PartialEq, Debug, Clone, Copy, Default)]
pub enum Animate<T: Copy + Clone> {
	#[default]
	None,
	Replay(T),
	Repeat(T),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Outdated<TComponent: Component> {
	pub entity: Entity,
	pub component: TComponent,
}

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

	pub fn set_render_layer<TUiCamera>(&self, render_layers: &mut RenderLayers)
	where
		TUiCamera: StaticRenderLayers,
	{
		*render_layers = TUiCamera::render_layers()
	}
}

#[derive(Component, Debug, PartialEq, Clone)]
pub struct NoTarget;

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub enum AssetModel {
	#[default]
	None,
	Path(String),
}

impl AssetModel {
	pub fn path(path: &str) -> AssetModel {
		AssetModel::Path(path.to_owned())
	}

	pub fn flip_on(self, name: Name) -> (Self, FlipHorizontally) {
		(self, FlipHorizontally::with(name))
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct Protected<TComponent: Component>(PhantomData<TComponent>);

impl<TComponent: Component> Default for Protected<TComponent> {
	fn default() -> Self {
		Self(PhantomData)
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

		let node = UiNodeFor::<()>::with(Entity::from_raw(100));
		let mut render_layers = RenderLayers::default();

		node.set_render_layer::<_T>(&mut render_layers);

		assert_eq!(RenderLayers::layer(42), render_layers);
	}
}
