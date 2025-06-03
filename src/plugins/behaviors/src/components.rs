pub mod ground_target;
pub mod skill_behavior;

pub(crate) mod anchor;
pub(crate) mod movement;
pub(crate) mod set_to_move_forward;
pub(crate) mod when_traveled_insert;

use bevy::prelude::*;
use common::{
	tools::UnitsPerSecond,
	traits::{animation::Animation, handles_orientation::Face},
};
use std::{fmt::Debug, time::Duration};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Always;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Once;

#[derive(Component, Debug, PartialEq)]
pub struct OverrideFace(pub Face);

#[derive(Component, Debug, PartialEq)]
pub struct SetFace(pub Face);

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Fast,
	Slow,
}

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub struct MovementConfig {
	pub speed: UnitsPerSecond,
	pub animation: Option<Animation>,
}

#[derive(Component, Debug, PartialEq)]
pub struct Chase(pub Entity);

#[derive(Component, Debug, PartialEq)]
pub struct Attack(pub Entity);

#[derive(Component, Debug, PartialEq)]
pub(crate) struct OnCoolDown(pub Duration);
