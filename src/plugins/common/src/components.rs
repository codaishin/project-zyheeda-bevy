pub mod effected_by;

use bevy::{
	ecs::{component::Component, entity::Entity},
	math::Vec3,
};
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
	Main,
	Off,
}

#[derive(Component)]
pub struct GroundOffset(pub Vec3);

#[derive(Component, Default)]
pub struct Player;

impl Player {
	pub const MODEL_PATH: &'static str = "models/player.glb";
}

#[derive(Component, Debug, PartialEq)]
pub struct Immobilized;

#[derive(Component, Debug, PartialEq)]
pub struct Idle;

#[derive(Component)]
pub struct Dummy;

#[derive(Component, Debug, PartialEq)]
pub struct Health {
	pub current: i16,
	pub max: i16,
}

impl Health {
	pub fn new(value: i16) -> Self {
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
