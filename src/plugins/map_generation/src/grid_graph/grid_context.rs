use crate::traits::{grid_min::GridMin, key_mapper::KeyMapper};
use bevy::prelude::*;
use common::errors::{Error, Level};
use macros::{InRange, new_valid};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GridContext {
	pub(crate) cell_count_x: CellCount,
	pub(crate) cell_count_z: CellCount,
	pub(crate) cell_distance: CellDistance,
}

impl GridContext {
	const DEFAULT: Self = Self {
		cell_count_x: new_valid!(CellCount, 2),
		cell_count_z: new_valid!(CellCount, 2),
		cell_distance: new_valid!(CellDistance, 1.),
	};
}

impl Default for GridContext {
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl GridMin for GridContext {
	fn grid_min(&self) -> Vec3 {
		let x = ((*self.cell_count_x - 1) as f32 * *self.cell_distance) / 2.;
		let z = ((*self.cell_count_z - 1) as f32 * *self.cell_distance) / 2.;

		Vec3::new(-x, 0., -z)
	}
}

impl KeyMapper for GridContext {
	fn key_for(&self, translation: Vec3) -> Option<(u32, u32)> {
		let start = self.grid_min();
		let Vec3 { x, z, .. } = translation - start;
		let x = (x / *self.cell_distance).round();
		let z = (z / *self.cell_distance).round();

		if x < 0. || z < 0. {
			return None;
		}

		Some((x as u32, z as u32))
	}
}

#[derive(Debug, PartialEq, Clone, Copy, InRange)]
#[in_range(low = > 0., high = < f32::INFINITY)]
pub(crate) struct CellDistance(f32);

impl CellDistance {
	pub(crate) fn divided_by(self, divisor: u8) -> Result<Self, DividedToZero> {
		Self::try_from(self.0 / divisor as f32).map_err(|_| DividedToZero {
			from: self.0,
			divisor,
		})
	}
}

#[derive(Debug, PartialEq, Clone, Copy, InRange)]
#[in_range(low = > 0, high = u32::MAX)]
pub(crate) struct CellCount(u32);

impl CellCount {
	pub(crate) fn multiply_by(self, factor: u8) -> Result<Self, MultipliedTooHigh> {
		let Some(product) = self.0.checked_mul(factor as u32) else {
			return Err(MultipliedTooHigh {
				from: self.0,
				factor,
			});
		};

		Ok(Self(product))
	}

	pub(crate) fn try_from_max_index(i: u32) -> Option<Self> {
		let count = i.checked_add(1)?;
		Some(Self(count))
	}
}

#[derive(Debug, PartialEq)]
pub struct DividedToZero {
	pub(crate) from: f32,
	pub(crate) divisor: u8,
}

impl From<DividedToZero> for Error {
	fn from(DividedToZero { from, divisor }: DividedToZero) -> Self {
		Error::Single {
			msg: format!("dividing cell distance {from} by {divisor} resulted in 0.",),
			lvl: Level::Error,
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct MultipliedTooHigh {
	pub(crate) from: u32,
	pub(crate) factor: u8,
}

impl From<MultipliedTooHigh> for Error {
	fn from(MultipliedTooHigh { from, factor }: MultipliedTooHigh) -> Self {
		Error::Single {
			msg: format!("multiplying cell count {from} by {factor} was invalid",),
			lvl: Level::Error,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;
	use zyheeda_core::errors::NotInRange;

	const CELL_COUNT_2: CellCount = new_valid!(CellCount, 2);
	const CELL_COUNT_3: CellCount = new_valid!(CellCount, 3);
	const CELL_COUNT_6: CellCount = new_valid!(CellCount, 6);

	const CELL_DISTANCE_1: CellDistance = new_valid!(CellDistance, 1.);
	const CELL_DISTANCE_2: CellDistance = new_valid!(CellDistance, 2.);
	const CELL_DISTANCE_3: CellDistance = new_valid!(CellDistance, 3.);

	#[test_case(CELL_COUNT_2, CELL_COUNT_2, CELL_DISTANCE_1, Vec3::new(-0.5, 0., -0.5); "grid 2 by 2 with distance 1")]
	#[test_case(CELL_COUNT_3, CELL_COUNT_3, CELL_DISTANCE_1, Vec3::new(-1., 0., -1.); "grid 3 by 3 with distance 1")]
	#[test_case(CELL_COUNT_2, CELL_COUNT_2, CELL_DISTANCE_2, Vec3::new(-1., 0., -1.); "grid 2 by 2 with distance 2")]
	#[test_case(CELL_COUNT_3, CELL_COUNT_3, CELL_DISTANCE_2, Vec3::new(-2., 0., -2.); "grid 3 by 3 with distance 2")]
	#[test_case(CELL_COUNT_3, CELL_COUNT_6, CELL_DISTANCE_3, Vec3::new(-3., 0., -7.5); "grid 3 by 6 with distance 3")]
	fn get_min(
		cell_count_x: CellCount,
		cell_count_z: CellCount,
		cell_distance: CellDistance,
		result: Vec3,
	) {
		let context = GridContext {
			cell_count_x,
			cell_count_z,
			cell_distance,
		};

		let min = context.grid_min();

		assert_eq!(result, min);
	}

	#[test_case(CELL_COUNT_2, CELL_COUNT_2, CELL_DISTANCE_1, Vec3::new(0.5, 0., -0.5), Some((1, 0)); "grid 2 by 2 with distance 1")]
	#[test_case(CELL_COUNT_3, CELL_COUNT_3, CELL_DISTANCE_1, Vec3::new(0., 0., 1.), Some((1, 2)); "grid 3 by 3 with distance 1")]
	#[test_case(CELL_COUNT_2, CELL_COUNT_2, CELL_DISTANCE_2, Vec3::new(1., 0., -1.5), Some((1, 0)); "grid 2 by 2 with distance 2")]
	#[test_case(CELL_COUNT_3, CELL_COUNT_3, CELL_DISTANCE_2, Vec3::new(0., 0., 2.), Some((1, 2)); "grid 3 by 3 with distance 2")]
	#[test_case(CELL_COUNT_2, CELL_COUNT_2, CELL_DISTANCE_1, Vec3::new(0.4, 0., -0.7), Some((1, 0)); "grid 2 by 2 with distance 1 rounded")]
	#[test_case(CELL_COUNT_2, CELL_COUNT_2, CELL_DISTANCE_2, Vec3::new(0.8, 0., -1.4), Some((1, 0)); "grid 2 by 2 with distance 2 rounded")]
	#[test_case(CELL_COUNT_2, CELL_COUNT_2, CELL_DISTANCE_1, Vec3::new(0.5, 0., -1.5), None; "grid 2 by 2 with distance 1 error")]
	#[test_case(CELL_COUNT_2, CELL_COUNT_2, CELL_DISTANCE_2, Vec3::new(0.5, 0., -3.), None; "grid 2 by 2 with distance 2 error")]
	fn get_key(
		cell_count_x: CellCount,
		cell_count_z: CellCount,
		cell_distance: CellDistance,
		target: Vec3,
		result: Option<(u32, u32)>,
	) {
		let context = GridContext {
			cell_count_x,
			cell_count_z,
			cell_distance,
		};

		let key = context.key_for(target);

		assert_eq!(result, key);
	}

	#[test]
	fn divide_valid() {
		let distance = new_valid!(CellDistance, 1.);

		let divided = distance.divided_by(10);

		assert_eq!(Ok(new_valid!(CellDistance, 0.1)), divided);
	}

	#[test]
	fn divide_invalid_to_zero() -> Result<(), NotInRange<f32>> {
		let super_low = (0.0_f32).next_up();
		let distance = CellDistance::try_from(super_low)?;

		let divided = distance.divided_by(10);

		assert_eq!(
			Err(DividedToZero {
				from: super_low,
				divisor: 10,
			}),
			divided
		);
		Ok(())
	}

	#[test]
	fn multiply_valid() {
		let count = new_valid!(CellCount, 1);

		let product = count.multiply_by(10);

		assert_eq!(Ok(new_valid!(CellCount, 10)), product);
	}

	#[test]
	fn multiply_invalid_overflow() {
		let count = new_valid!(CellCount, 4294967294); // u32::MAX - 1

		let product = count.multiply_by(2);

		assert_eq!(
			Err(MultipliedTooHigh {
				from: 4294967294,
				factor: 2,
			}),
			product
		);
	}
}
