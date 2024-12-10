pub mod cam_orbit;
pub mod ground_targeted_aoe;
pub mod projectile;
pub mod shield;
pub mod void_beam;

pub(crate) mod constant_movement;

use crate::traits::RemoveComponent;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::test_tools::utils::ApproxEqual;
use std::{fmt::Debug, marker::PhantomData, time::Duration};

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum Face {
	#[default]
	Cursor,
	Entity(Entity),
	Translation(Vec3),
}

#[derive(Component, Debug, PartialEq)]
pub struct OverrideFace(pub Face);

#[derive(Component, Debug, PartialEq)]
pub struct SetFace(pub Face);

#[derive(PartialEq, Debug)]
pub struct PositionBased;

#[derive(PartialEq, Debug)]
pub struct VelocityBased;

#[derive(Component, Clone, PartialEq, Debug, Default)]
pub struct Movement<TMovement> {
	pub(crate) target: Vec3,
	pub(crate) cleanup: Option<fn(&mut EntityCommands)>,
	phantom_data: PhantomData<TMovement>,
}

impl<TMovement> Movement<TMovement> {
	pub fn to(target: Vec3) -> Self {
		Self {
			target,
			cleanup: None,
			phantom_data: PhantomData,
		}
	}

	pub fn remove_on_cleanup<TBundle: Bundle>(self) -> Self {
		Self {
			target: self.target,
			cleanup: Some(TBundle::get_remover()),
			phantom_data: self.phantom_data,
		}
	}
}

impl<TMovement> ApproxEqual<f32> for Movement<TMovement> {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.target.approx_equal(&other.target, tolerance) && self.cleanup == other.cleanup
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct Chase(pub Entity);

#[derive(Component, Debug, PartialEq)]
pub struct Attack(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attacker(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Target(pub Entity);

#[derive(Component, Debug, PartialEq)]
pub(crate) struct OnCoolDown(pub Duration);
