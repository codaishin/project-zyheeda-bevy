use crate::traits::{grid_start::GridStart, key_mapper::KeyMapper};
use bevy::prelude::*;
use common::errors::{Error, Level};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GridContext(GridDefinition);

impl Default for GridContext {
	fn default() -> Self {
		Self(GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: 1,
		})
	}
}

impl TryFrom<GridDefinition> for GridContext {
	type Error = GridDefinitionError;

	fn try_from(config: GridDefinition) -> Result<Self, Self::Error> {
		if config.cell_count_x == 0 || config.cell_count_z == 0 {
			return Err(GridDefinitionError::CellCountZero);
		}

		if config.cell_distance == 0 {
			return Err(GridDefinitionError::CellDistanceZero);
		}

		Ok(Self(config))
	}
}

impl GridStart for GridContext {
	fn grid_min(&self) -> Vec3 {
		let Self(d) = self;
		let cell_distance = d.cell_distance as usize;
		let x = ((d.cell_count_x - 1) * cell_distance) as f32 / 2.;
		let z = ((d.cell_count_z - 1) * cell_distance) as f32 / 2.;

		Vec3::new(-x, 0., -z)
	}
}

impl KeyMapper for GridContext {
	fn key_for(&self, translation: Vec3) -> (i32, i32) {
		let start = self.grid_min();
		let cell_distance = self.0.cell_distance as f32;
		let Vec3 { x, z, .. } = translation - start;

		(
			(x / cell_distance).round() as i32,
			(z / cell_distance).round() as i32,
		)
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct GridDefinition {
	pub(crate) cell_count_x: usize,
	pub(crate) cell_count_z: usize,
	pub(crate) cell_distance: u8,
}

#[derive(Debug, PartialEq)]
pub(crate) enum GridDefinitionError {
	CellCountZero,
	CellDistanceZero,
}

impl From<GridDefinitionError> for Error {
	fn from(error: GridDefinitionError) -> Self {
		match error {
			GridDefinitionError::CellCountZero => Error {
				msg: "Grid definition is empty".to_owned(),
				lvl: Level::Error,
			},
			GridDefinitionError::CellDistanceZero => Error {
				msg: "Grid cell distance is zero".to_owned(),
				lvl: Level::Error,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test]
	fn from_definition() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: 1,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Ok(GridContext(definition)), context)
	}

	#[test]
	fn from_definition_no_cells_when_x_zero() {
		let definition = GridDefinition {
			cell_count_x: 0,
			cell_count_z: 1,
			cell_distance: 1,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellCountZero), context)
	}

	#[test]
	fn from_definition_no_cells_when_z_zero() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 0,
			cell_distance: 1,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellCountZero), context)
	}

	#[test]
	fn from_definition_no_distance_when() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: 0,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellDistanceZero), context)
	}

	#[test]
	fn get_start_1_1() -> Result<(), GridDefinitionError> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: 1,
		})?;

		let start = context.grid_min();

		assert_eq!(Vec3::default(), start);
		Ok(())
	}

	#[test_case(2, 2, 1, Vec3::new(-0.5, 0., -0.5); "grid 2 by 2 with distance 1")]
	#[test_case(3, 3, 1, Vec3::new(-1., 0., -1.); "grid 3 by 3 with distance 1")]
	#[test_case(2, 2, 2, Vec3::new(-1., 0., -1.); "grid 2 by 2 with distance 2")]
	#[test_case(3, 3, 2, Vec3::new(-2., 0., -2.); "grid 3 by 3 with distance 2")]
	fn get_min(
		cell_count_x: usize,
		cell_count_z: usize,
		cell_distance: u8,
		result: Vec3,
	) -> Result<(), GridDefinitionError> {
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
	fn get_key_1_by_1() -> Result<(), GridDefinitionError> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: 1,
		})?;

		let key = context.key_for(Vec3::ZERO);

		assert_eq!((0, 0), key);
		Ok(())
	}

	#[test_case(2, 2, 1, Vec3::new(0.5, 0., -1.5), (1, -1); "grid 2 by 2 with distance 1")]
	#[test_case(3, 3, 1, Vec3::new(0., 0., 1.), (1, 2); "grid 3 by 3 with distance 1")]
	#[test_case(2, 2, 2, Vec3::new(1., 0., -3.), (1, -1); "grid 2 by 2 with distance 2")]
	#[test_case(3, 3, 2, Vec3::new(0., 0., 2.), (1, 2); "grid 3 by 3 with distance 2")]
	#[test_case(2, 2, 1, Vec3::new(0.4, 0., -1.4), (1, -1); "grid 2 by 2 with distance 1 rounded")]
	#[test_case(2, 2, 2, Vec3::new(0.8, 0., -2.8), (1, -1); "grid 2 by 2 with distance 2 rounded")]
	fn get_key(
		cell_count_x: usize,
		cell_count_z: usize,
		cell_distance: u8,
		target: Vec3,
		result: (i32, i32),
	) -> Result<(), GridDefinitionError> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x,
			cell_count_z,
			cell_distance,
		})?;

		let key = context.key_for(target);

		assert_eq!(result, key);
		Ok(())
	}
}
