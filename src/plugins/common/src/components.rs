pub mod essence;
pub mod flip;

use bevy::prelude::*;
use flip::FlipHorizontally;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone)]
pub struct Swap<T1, T2>(pub T1, pub T2);

#[derive(Component, Debug, PartialEq)]
pub struct Collection<TElement>(pub Vec<TElement>);

impl<TElement> Collection<TElement> {
	pub fn new<const N: usize>(elements: [TElement; N]) -> Self {
		Self(elements.into())
	}
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Side {
	Right,
	Left,
}

#[derive(Component)]
pub struct GroundOffset(pub Vec3);

#[derive(Component, Debug, PartialEq)]
pub struct Immobilized;

#[derive(Component, Debug, PartialEq)]
pub struct Idle;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Health {
	pub current: f32,
	pub max: f32,
}

impl Health {
	pub fn new(value: f32) -> Self {
		Self {
			current: value,
			max: value,
		}
	}
}

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
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
pub struct OwnedBy<TOwner> {
	pub owner: Entity,
	owner_type: PhantomData<TOwner>,
}

impl<T> OwnedBy<T> {
	pub fn with(owner: Entity) -> Self {
		Self {
			owner,
			owner_type: PhantomData,
		}
	}
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct NoTarget;

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
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
