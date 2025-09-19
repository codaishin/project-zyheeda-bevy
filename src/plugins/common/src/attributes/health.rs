use serde::{Deserialize, Serialize};

use crate::traits::accessors::get::Property;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
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

impl Property for Health {
	type TValue<'a> = Self;
}
