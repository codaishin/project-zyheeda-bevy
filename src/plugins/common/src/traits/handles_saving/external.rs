use crate::impl_savable_self_non_priority;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

impl_savable_self_non_priority!(Transform, Name, Velocity);
