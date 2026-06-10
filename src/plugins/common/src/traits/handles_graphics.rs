use crate::traits::accessors::get::{TryGetContext, TryGetContextMut};
use bevy::prelude::*;
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

pub trait UiCamera {
	type TUiCamera: Component;
}

pub trait FirstPassCamera {
	type TFirstPassCamera: Component;
}

pub trait WorldCameras {
	type TWorldCameras: Component;
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
