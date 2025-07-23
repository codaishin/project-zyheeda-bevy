use crate::traits::{grid_min::GridMin, key_mapper::KeyMapper};
use bevy::prelude::*;
use common::errors::{Error, Level};
use macros::{InRange, new_valid};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GridContext(pub(super) GridDefinition);

impl Default for GridContext {
	fn default() -> Self {
		Self(GridDefinition::DEFAULT)
	}
}

impl TryFrom<GridDefinition> for GridContext {
	type Error = CellCountZero;

	fn try_from(config: GridDefinition) -> Result<Self, Self::Error> {
		if config.cell_count_x == 0 || config.cell_count_z == 0 {
			return Err(CellCountZero);
		}

		Ok(Self(config))
	}
}

impl GridMin for GridContext {
	fn grid_min(&self) -> Vec3 {
		let Self(d) = self;
		let x = ((d.cell_count_x - 1) as f32 * *d.cell_distance) / 2.;
		let z = ((d.cell_count_z - 1) as f32 * *d.cell_distance) / 2.;

		Vec3::new(-x, 0., -z)
	}
}

impl KeyMapper for GridContext {
	fn key_for(&self, translation: Vec3) -> Option<(usize, usize)> {
		let Self(definition) = self;
		let start = self.grid_min();
		let Vec3 { x, z, .. } = translation - start;
		let x = (x / *definition.cell_distance).round();
		let z = (z / *definition.cell_distance).round();

		if x < 0. || z < 0. {
			return None;
		}

		Some((x as usize, z as usize))
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct GridDefinition {
	pub(crate) cell_count_x: usize,
	pub(crate) cell_count_z: usize,
	pub(crate) cell_distance: CellDistance,
}

impl GridDefinition {
	const DEFAULT: Self = Self {
		cell_count_x: 1,
		cell_count_z: 1,
		cell_distance: new_valid!(CellDistance, 1.),
	};
}

#[derive(Debug, PartialEq, Clone, Copy, InRange)]
#[in_range(low = > 0., high = < f32::INFINITY)]
pub(crate) struct CellDistance(f32);

impl CellDistance {
	pub(crate) fn dived_by(self, divisor: u8) -> Result<Self, DividedToZero> {
		Self::try_from(self.0 / divisor as f32).map_err(|_| DividedToZero {
			from: self.0,
			divisor,
		})
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
pub(crate) struct CellCountZero;

impl From<CellCountZero> for Error {
	fn from(_: CellCountZero) -> Self {
		Error::Single {
			msg: "Grid definition is empty".to_owned(),
			lvl: Level::Error,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;
	use zyheeda_core::errors::NotInRange;

	#[test]
	fn from_definition() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: new_valid!(CellDistance, 1.),
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Ok(GridContext(definition)), context)
	}

	#[test]
	fn from_definition_no_cells_when_x_zero() {
		let definition = GridDefinition {
			cell_count_x: 0,
			cell_count_z: 1,
			cell_distance: new_valid!(CellDistance, 1.),
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(CellCountZero), context)
	}

	#[test]
	fn from_definition_no_cells_when_z_zero() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 0,
			cell_distance: new_valid!(CellDistance, 1.),
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(CellCountZero), context)
	}

	#[test]
	fn get_start_1_1() -> Result<(), CellCountZero> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: new_valid!(CellDistance, 1.),
		})?;

		let start = context.grid_min();

		assert_eq!(Vec3::default(), start);
		Ok(())
	}

	#[test_case(2, 2, new_valid!(CellDistance, 1.), Vec3::new(-0.5, 0., -0.5); "grid 2 by 2 with distance 1")]
	#[test_case(3, 3, new_valid!(CellDistance, 1.), Vec3::new(-1., 0., -1.); "grid 3 by 3 with distance 1")]
	#[test_case(2, 2, new_valid!(CellDistance, 2.), Vec3::new(-1., 0., -1.); "grid 2 by 2 with distance 2")]
	#[test_case(3, 3, new_valid!(CellDistance, 2.), Vec3::new(-2., 0., -2.); "grid 3 by 3 with distance 2")]
	#[test_case(3, 6, new_valid!(CellDistance, 3.), Vec3::new(-3., 0., -7.5); "grid 3 by 6 with distance 3")]
	fn get_min(
		cell_count_x: usize,
		cell_count_z: usize,
		cell_distance: CellDistance,
		result: Vec3,
	) -> Result<(), CellCountZero> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x,
			cell_count_z,
			cell_distance,
		})?;

		let min = context.grid_min();

		assert_eq!(result, min);
		Ok(())
	}

	#[test]
	fn get_key_1_by_1() -> Result<(), CellCountZero> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: new_valid!(CellDistance, 1.),
		})?;

		let key = context.key_for(Vec3::ZERO);

		assert_eq!(Some((0, 0)), key);
		Ok(())
	}

	#[test_case(2, 2, new_valid!(CellDistance, 1.), Vec3::new(0.5, 0., -0.5), Some((1, 0)); "grid 2 by 2 with distance 1")]
	#[test_case(3, 3, new_valid!(CellDistance, 1.), Vec3::new(0., 0., 1.), Some((1, 2)); "grid 3 by 3 with distance 1")]
	#[test_case(2, 2, new_valid!(CellDistance, 2.), Vec3::new(1., 0., -1.5), Some((1, 0)); "grid 2 by 2 with distance 2")]
	#[test_case(3, 3, new_valid!(CellDistance, 2.), Vec3::new(0., 0., 2.), Some((1, 2)); "grid 3 by 3 with distance 2")]
	#[test_case(2, 2, new_valid!(CellDistance, 1.), Vec3::new(0.4, 0., -0.7), Some((1, 0)); "grid 2 by 2 with distance 1 rounded")]
	#[test_case(2, 2, new_valid!(CellDistance, 2.), Vec3::new(0.8, 0., -1.4), Some((1, 0)); "grid 2 by 2 with distance 2 rounded")]
	#[test_case(2, 2, new_valid!(CellDistance, 1.), Vec3::new(0.5, 0., -1.5), None; "grid 2 by 2 with distance 1 error")]
	#[test_case(2, 2, new_valid!(CellDistance, 2.), Vec3::new(0.5, 0., -3.), None; "grid 2 by 2 with distance 2 error")]
	fn get_key(
		cell_count_x: usize,
		cell_count_z: usize,
		cell_distance: CellDistance,
		target: Vec3,
		result: Option<(usize, usize)>,
	) -> Result<(), CellCountZero> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x,
			cell_count_z,
			cell_distance,
		})?;

		let key = context.key_for(target);

		assert_eq!(result, key);
		Ok(())
	}

	#[test]
	fn divide_valid() {
		let distance = new_valid!(CellDistance, 1.);

		let divided = distance.dived_by(10);

		assert_eq!(Ok(new_valid!(CellDistance, 0.1)), divided);
	}

	#[test]
	fn divide_valid_to_zero() -> Result<(), NotInRange<f32>> {
		let super_low = (0.0_f32).next_up();
		let distance = CellDistance::try_from(super_low)?;

		let divided = distance.dived_by(10);

		assert_eq!(
			Err(DividedToZero {
				from: super_low,
				divisor: 10,
			}),
			divided
		);
		Ok(())
	}
}
