use bevy::prelude::*;
use common::tools::vec_not_nan::VecNotNan;
use std::collections::HashMap;
use zyheeda_core::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct Impacted {
	pub(crate) impact_points: HashMap<VecNotNan<3>, F32FinitePositive>,
}
