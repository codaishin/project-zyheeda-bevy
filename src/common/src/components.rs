use crate::tools::UnitsPerSecond;
use bevy::{
	ecs::{component::Component, entity::Entity},
	math::Vec3,
};
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
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

#[derive(Component, Default)]
pub struct Player;

#[derive(Component)]
pub struct Dummy;

#[derive(Component)]
pub struct VoidSphere;

#[derive(Component, Clone)]
pub enum VoidSpherePart {
	Core,
	RingA(UnitsPerSecond),
	RingB(UnitsPerSecond),
}

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

#[derive(Component, PartialEq, Debug)]
pub struct ColliderRoot(pub Entity);

#[derive(Component)]
pub struct DealsDamage(pub i16);

pub struct Plasma;

#[derive(Component, Default)]
pub struct Projectile<T> {
	pub direction: Vec3,
	pub range: f32,
	phantom_data: PhantomData<T>,
}

impl<T> Projectile<T> {
	pub fn new(direction: Vec3, range: f32) -> Self {
		Self {
			direction,
			range,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Component, PartialEq, Debug, Clone, Copy, Default)]
pub enum Animate<T: Copy + Clone> {
	#[default]
	None,
	Replay(T),
	Repeat(T),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Outdated<TComponent: Component> {
	pub entity: Entity,
	pub component: TComponent,
}
