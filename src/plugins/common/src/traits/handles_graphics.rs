use crate::traits::accessors::get::{GetContext, GetContextMut, TryGetContext, TryGetContextMut};
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::EntityKey;
use std::ops::{Deref, DerefMut};

pub trait HandlesGraphics {
	type THighlight: for<'c> TryGetContext<Visual, TContext<'c>: GetHighlight>;
	type THighlightMut: for<'c> TryGetContextMut<Visual, TContext<'c>: SetHighlight>;
	type TRolesMut: for<'c> TryGetContextMut<HasNoRole, TContext<'c>: SetRole>;
}

#[derive(EntityKey)]
pub struct Visual {
	pub entity: Entity,
}

pub trait GetHighlight {
	fn get_highlight(&self) -> Highlight;
}

impl<T> GetHighlight for T
where
	T: Deref<Target: GetHighlight>,
{
	fn get_highlight(&self) -> Highlight {
		self.deref().get_highlight()
	}
}

pub trait SetHighlight: GetHighlight {
	fn set_highlight(&mut self, highlight: Highlight);
}

impl<T> SetHighlight for T
where
	T: DerefMut<Target: SetHighlight>,
{
	fn set_highlight(&mut self, highlight: Highlight) {
		self.deref_mut().set_highlight(highlight)
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Highlight {
	None,
	Interacting,
}

pub trait HandlesCameras {
	type TCamera: for<'c> GetContext<CameraHandle, TContext<'c>: CameraTransform>;
	type TCameraMut: SystemParam
		+ for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>
		+ for<'c> GetContextMut<CameraHandle, TContext<'c>: ScreenPosition>
		+ for<'c> GetContextMut<CameraHandle, TContext<'c>: CameraTransformMut>;
}

pub struct CameraHandle;

pub trait RenderUi {
	fn render_ui(&mut self, ui: Entity);
}

pub trait ScreenPosition {
	fn screen_position(&self, translation: Vec3) -> Option<Vec2>;
}

pub trait CameraTransform {
	fn camera_transform(&self) -> &'_ Transform;
}

impl<T> CameraTransform for T
where
	T: Deref<Target: CameraTransform>,
{
	fn camera_transform(&self) -> &'_ Transform {
		self.deref().camera_transform()
	}
}

pub trait CameraTransformMut: CameraTransform {
	fn camera_transform_mut(&mut self) -> &'_ mut Transform;
}

impl<T> CameraTransformMut for T
where
	T: DerefMut<Target: CameraTransformMut>,
{
	fn camera_transform_mut(&mut self) -> &'_ mut Transform {
		self.deref_mut().camera_transform_mut()
	}
}

#[derive(EntityKey)]
pub struct HasNoRole {
	pub entity: Entity,
}

pub trait SetRole {
	fn set_role(&mut self, role: Role);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Role {
	Player,
	Enemy,
}
