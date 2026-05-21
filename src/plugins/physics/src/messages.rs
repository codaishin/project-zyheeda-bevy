use bevy::prelude::*;
use common::traits::handles_physics::TimeOfImpact;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray(pub Ray3d, pub TimeOfImpact);

#[derive(Message, Debug, PartialEq, Clone, Copy)]
pub struct RayEvent(pub Entity, pub Ray);
