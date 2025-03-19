use crate::traits::{grid_min::GridMin, key_mapper::KeyMapper};
use bevy::prelude::*;
use common::errors::{Error, Level};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GridContext(pub(super) GridDefinition);

impl Default for GridContext {
	fn default() -> Self {
		Self(GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: 1.,
		})
	}
}

impl TryFrom<GridDefinition> for GridContext {
	type Error = GridDefinitionError;

	fn try_from(config: GridDefinition) -> Result<Self, Self::Error> {
		if config.cell_count_x == 0 || config.cell_count_z == 0 {
			return Err(GridDefinitionError::CellCountZero);
		}

		match config.cell_distance {
			0. => Err(GridDefinitionError::CellDistanceZero),
			d if d < 0. => Err(GridDefinitionError::CellDistanceNegative),
			d if d.is_infinite() => Err(GridDefinitionError::CellDistanceInfinite),
			d if d.is_nan() => Err(GridDefinitionError::CellDistanceNaN),
			_ => Ok(Self(config)),
		}
	}
}

impl GridMin for GridContext {
	fn grid_min(&self) -> Vec3 {
		let Self(d) = self;
		let x = ((d.cell_count_x - 1) as f32 * d.cell_distance) / 2.;
		let z = ((d.cell_count_z - 1) as f32 * d.cell_distance) / 2.;

		Vec3::new(-x, 0., -z)
	}
}

impl KeyMapper for GridContext {
	fn key_for(&self, translation: Vec3) -> Option<(usize, usize)> {
		let Self(definition) = self;
		let start = self.grid_min();
		let Vec3 { x, z, .. } = translation - start;
		let x = (x / definition.cell_distance).round();
		let z = (z / definition.cell_distance).round();

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
	pub(crate) cell_distance: f32,
}

#[derive(Debug, PartialEq)]
pub(crate) enum GridDefinitionError {
	CellCountZero,
	CellDistanceZero,
	CellDistanceNegative,
	CellDistanceNaN,
	CellDistanceInfinite,
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
			GridDefinitionError::CellDistanceNegative => Error {
				msg: "Grid cell distance is negative".to_owned(),
				lvl: Level::Error,
			},
			GridDefinitionError::CellDistanceNaN => Error {
				msg: "Grid cell distance is NaN".to_owned(),
				lvl: Level::Error,
			},
			GridDefinitionError::CellDistanceInfinite => Error {
				msg: "Grid cell distance is infinite".to_owned(),
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
			cell_distance: 1.,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Ok(GridContext(definition)), context)
	}

	#[test]
	fn from_definition_no_cells_when_x_zero() {
		let definition = GridDefinition {
			cell_count_x: 0,
			cell_count_z: 1,
			cell_distance: 1.,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellCountZero), context)
	}

	#[test]
	fn from_definition_no_cells_when_z_zero() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 0,
			cell_distance: 1.,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellCountZero), context)
	}

	#[test]
	fn from_definition_no_distance() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: 0.,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellDistanceZero), context)
	}

	#[test]
	fn from_definition_distance_not_a_number() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: f32::NAN,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellDistanceNaN), context)
	}

	#[test]
	fn from_definition_distance_negative() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: -1.,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellDistanceNegative), context)
	}

	#[test]
	fn from_definition_distance_infinite() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: f32::INFINITY,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellDistanceInfinite), context)
	}

	#[test]
	fn from_definition_distance_neg_infinite() {
		let definition = GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: f32::NEG_INFINITY,
		};

		let context = GridContext::try_from(definition);

		assert_eq!(Err(GridDefinitionError::CellDistanceNegative), context)
	}

	#[test]
	fn get_start_1_1() -> Result<(), GridDefinitionError> {
		let context = GridContext::try_from(GridDefinition {
			cell_count_x: 1,
			cell_count_z: 1,
			cell_distance: 1.,
		})?;

		let start = context.grid_min();

		assert_eq!(Vec3::default(), start);
		Ok(())
	}

	#[test_case(2, 2, 1., Vec3::new(-0.5, 0., -0.5); "grid 2 by 2 with distance 1")]
	#[test_case(3, 3, 1., Vec3::new(-1., 0., -1.); "grid 3 by 3 with distance 1")]
	#[test_case(2, 2, 2., Vec3::new(-1., 0., -1.); "grid 2 by 2 with distance 2")]
	#[test_case(3, 3, 2., Vec3::new(-2., 0., -2.); "grid 3 by 3 with distance 2")]
	#[test_case(3, 6, 10. / 3., Vec3::new(-10. / 3., 0., -5. -10. / 3.); "grid 3 by 6 with distance ten third")]
	fn get_min(
		cell_count_x: usize,
		cell_count_z: usize,
		cell_distance: f32,
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
			cell_distance: 1.,
		})?;

		let key = context.key_for(Vec3::ZERO);

		assert_eq!(Some((0, 0)), key);
		Ok(())
	}

	#[test_case(2, 2, 1., Vec3::new(0.5, 0., -0.5), Some((1, 0)); "grid 2 by 2 with distance 1")]
	#[test_case(3, 3, 1., Vec3::new(0., 0., 1.), Some((1, 2)); "grid 3 by 3 with distance 1")]
	#[test_case(2, 2, 2., Vec3::new(1., 0., -1.5), Some((1, 0)); "grid 2 by 2 with distance 2")]
	#[test_case(3, 3, 2., Vec3::new(0., 0., 2.), Some((1, 2)); "grid 3 by 3 with distance 2")]
	#[test_case(2, 2, 1., Vec3::new(0.4, 0., -0.7), Some((1, 0)); "grid 2 by 2 with distance 1 rounded")]
	#[test_case(2, 2, 2., Vec3::new(0.8, 0., -1.4), Some((1, 0)); "grid 2 by 2 with distance 2 rounded")]
	#[test_case(2, 2, 1., Vec3::new(0.5, 0., -1.5), None; "grid 2 by 2 with distance 1 error")]
	#[test_case(2, 2, 2., Vec3::new(0.5, 0., -3.), None; "grid 2 by 2 with distance 2 error")]
	fn get_key(
		cell_count_x: usize,
		cell_count_z: usize,
		cell_distance: f32,
		target: Vec3,
		result: Option<(usize, usize)>,
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
