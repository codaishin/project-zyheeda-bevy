pub(crate) mod agent;
pub(crate) mod corridor;
pub(crate) mod half_offset_cell;
pub(crate) mod parsed_color;

use crate::{
	cell_grid_size::CellGridSize,
	components::map::{
		Map,
		cells::{half_offset_cell::HalfOffsetCell, parsed_color::ParsedColor},
	},
	grid_graph::grid_context::CellCount,
	traits::{
		map_cells_extra::{CellGridDefinition, MapCellsExtra},
		parse_map_image::ParseMapImage,
		pixels::{Layer, PixelBytesIterator},
	},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level, Unreachable},
	traits::thread_safe::ThreadSafe,
};
use std::{collections::HashMap, fmt::Display};

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
#[require(Map)]
pub(crate) struct MapCells<TCell>
where
	TCell: MapCellsExtra,
{
	pub(crate) definition: CellGridDefinition<TCell>,
	pub(crate) extra: TCell::TExtra,
}

impl<TCell> Default for MapCells<TCell>
where
	TCell: MapCellsExtra<TExtra: Default>,
{
	fn default() -> Self {
		Self {
			definition: CellGridDefinition::default(),
			extra: TCell::TExtra::default(),
		}
	}
}

impl<TCell, TImage> ParseMapImage<TImage> for MapCells<TCell>
where
	TImage: PixelBytesIterator,
	for<'a> TCell: ParseMapImage<ParsedColor, TParseError = Unreachable>
		+ Clone
		+ ThreadSafe
		+ Default
		+ MapCellsExtra,
{
	type TParseError = MapSizeError;
	type TLookup = TCell::TLookup;

	fn try_parse(image: &TImage, lookup: &Self::TLookup) -> Result<Self, MapSizeError> {
		let mut indices = (0, 0);
		let mut cells = HashMap::default();

		for (UVec3 { x, y, .. }, bytes) in image.iter_pixel_bytes(Layer(0)) {
			indices.0 = u32::max(x, indices.0);
			indices.1 = u32::max(y, indices.1);

			let Ok(cell) = TCell::try_parse(&ParsedColor::parse(bytes), lookup);
			cells.insert((x, y), cell);
		}

		if cells.is_empty() {
			return Err(MapSizeError::Empty);
		}
		let size = CellGridSize {
			x: CellCount::try_from_max_index(indices.0).ok_or(MapSizeError::Indices {
				x: indices.0,
				z: indices.1,
			})?,
			z: CellCount::try_from_max_index(indices.1).ok_or(MapSizeError::Indices {
				x: indices.0,
				z: indices.1,
			})?,
		};

		let definition = CellGridDefinition {
			size,
			cells: CellGrid(cells),
		};

		Ok(Self {
			extra: TCell::TExtra::from(&definition),
			definition,
		})
	}
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct CellGrid<TCell>(pub(crate) HashMap<(u32, u32), TCell>);

impl<TCell> Default for CellGrid<TCell> {
	fn default() -> Self {
		Self(HashMap::default())
	}
}

impl<TCells, TCell> From<TCells> for CellGrid<TCell>
where
	TCells: IntoIterator<Item = ((u32, u32), TCell)>,
{
	fn from(cells: TCells) -> Self {
		Self(HashMap::from_iter(cells))
	}
}

impl<TCell> From<&CellGridDefinition<TCell>> for CellGrid<HalfOffsetCell<TCell>>
where
	TCell: Clone + Default,
{
	fn from(CellGridDefinition { size, cells }: &CellGridDefinition<TCell>) -> Self {
		let CellGrid(cells) = cells;

		let mut half_offset_cells = HashMap::default();
		for x in 1..*size.x {
			for z in 1..*size.z {
				let directions = HalfOffsetCell::directions(x, z).map(|(x, z, dir)| {
					let cell = cells.get(&(x, z)).cloned().unwrap_or_default();
					(dir, cell)
				});
				half_offset_cells.insert((x - 1, z - 1), HalfOffsetCell::from(directions));
			}
		}

		Self(half_offset_cells)
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum MapSizeError {
	Empty,
	Indices { x: u32, z: u32 },
}

impl Display for MapSizeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Minimum map size of 2x2 required, but map was {self:?}")
	}
}

impl From<MapSizeError> for Error {
	fn from(error: MapSizeError) -> Self {
		match error {
			MapSizeError::Empty => Self::Single {
				msg: String::from("map is empty"),
				lvl: Level::Error,
			},
			MapSizeError::Indices { x, z } => Self::Single {
				msg: format!(
					"indices too large x={x} and z={z} (max allowed {})",
					u32::MAX - 1
				),
				lvl: Level::Error,
			},
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum Direction {
	X,
	NegX,
	Z,
	NegZ,
}

impl Direction {
	pub(crate) const fn rotated_right(self, times: u8) -> Self {
		if times == 0 {
			return self;
		}

		if times > 1 {
			return self.rotated_right(1).rotated_right(times - 1);
		}

		match self {
			Direction::NegZ => Direction::NegX,
			Direction::NegX => Direction::Z,
			Direction::Z => Direction::X,
			Direction::X => Direction::NegZ,
		}
	}

	#[cfg(test)]
	pub(crate) const fn rotated_left(self) -> Self {
		match self {
			Direction::NegZ => Direction::X,
			Direction::NegX => Direction::NegZ,
			Direction::Z => Direction::NegX,
			Direction::X => Direction::Z,
		}
	}
}

impl From<Direction> for Dir3 {
	fn from(dir: Direction) -> Self {
		match dir {
			Direction::X => Dir3::X,
			Direction::NegX => Dir3::NEG_X,
			Direction::Z => Dir3::Z,
			Direction::NegZ => Dir3::NEG_Z,
		}
	}
}

#[cfg(test)]
mod test_parsing {
	use super::*;
	use crate::traits::pixels::{Bytes, Layer};
	use macros::new_valid;
	use mockall::{mock, predicate::eq};
	use std::vec::IntoIter;
	use testing::{Mock, simple_init};

	struct _Lookup;

	#[derive(TypePath, Debug, PartialEq, Clone, Default)]
	enum _Cell {
		#[default]
		Default,
		Value(ParsedColor),
	}

	impl ParseMapImage<ParsedColor> for _Cell {
		type TParseError = Unreachable;
		type TLookup = _Lookup;

		fn try_parse(parsed_color: &ParsedColor, _: &_Lookup) -> Result<Self, Self::TParseError> {
			Ok(_Cell::Value(*parsed_color))
		}
	}

	impl MapCellsExtra for _Cell {
		type TExtra = ();
	}

	struct _Image(Vec<(UVec3, Bytes<'static>)>);

	impl PixelBytesIterator for _Image {
		type TIter<'a>
			= IntoIter<(UVec3, Bytes<'a>)>
		where
			Self: 'a;

		fn iter_pixel_bytes(&self, _: Layer) -> Self::TIter<'_> {
			self.0.clone().into_iter()
		}
	}

	mock! {
		_Image {}
		impl PixelBytesIterator for _Image {
			type TIter<'a>
				= IntoIter<(UVec3, Bytes<'a>)>
			where
				Self: 'a;

			fn iter_pixel_bytes<'a>(&'a self, layer: Layer) -> IntoIter<(UVec3, Bytes<'a>)>;
		}
	}

	simple_init!(Mock_Image);

	#[test]
	fn empty() {
		let image = _Image(vec![]);

		let map = MapCells::<_Cell>::try_parse(&image, &_Lookup);

		assert_eq!(Err(MapSizeError::Empty), map);
	}

	#[test]
	fn too_large_y() {
		let image = _Image(vec![(uvec3(0, u32::MAX, 0), &[])]);

		let map = MapCells::<_Cell>::try_parse(&image, &_Lookup);

		assert_eq!(Err(MapSizeError::Indices { x: 0, z: u32::MAX }), map);
	}

	#[test]
	fn too_large_x() {
		let image = _Image(vec![(uvec3(u32::MAX, 0, 0), &[])]);

		let map = MapCells::<_Cell>::try_parse(&image, &_Lookup);

		assert_eq!(Err(MapSizeError::Indices { x: u32::MAX, z: 0 }), map);
	}

	#[test]
	fn too_large_x_and_z() {
		let image = _Image(vec![(uvec3(u32::MAX, u32::MAX, 0), &[])]);

		let map = MapCells::<_Cell>::try_parse(&image, &_Lookup);

		assert_eq!(
			Err(MapSizeError::Indices {
				x: u32::MAX,
				z: u32::MAX
			}),
			map
		);
	}

	#[test]
	fn use_layer_0() {
		let image = Mock_Image::new_mock(move |mock| {
			mock.expect_iter_pixel_bytes()
				.times(1)
				.with(eq(Layer(0)))
				.returning(|_| {
					vec![
						(uvec3(0, 0, 0), &[1_u8, 1_u8, 1_u8, 1_u8] as &[u8]),
						(uvec3(0, 1, 0), &[2_u8, 2_u8, 2_u8, 2_u8] as &[u8]),
						(uvec3(1, 0, 0), &[3_u8, 3_u8, 3_u8, 3_u8] as &[u8]),
						(uvec3(1, 1, 0), &[4_u8, 4_u8, 4_u8, 4_u8] as &[u8]),
					]
					.into_iter()
				});
		});

		_ = MapCells::<_Cell>::try_parse(&image, &_Lookup)
	}

	#[test]
	fn parse_pixels() -> Result<(), MapSizeError> {
		let image = _Image(vec![
			(uvec3(0, 0, 0), &[1, 1, 1, 1]),
			(uvec3(0, 1, 0), &[2, 2, 2, 2]),
			(uvec3(1, 0, 0), &[3, 3, 3, 3]),
			(uvec3(1, 1, 0), &[4, 4, 4, 4]),
		]);
		let map = MapCells::<_Cell>::try_parse(&image, &_Lookup)?;

		assert_eq!(
			MapCells {
				definition: CellGridDefinition {
					size: CellGridSize {
						x: new_valid!(CellCount, 2),
						z: new_valid!(CellCount, 2),
					},
					cells: CellGrid::from([
						((0, 0), _Cell::Value(ParsedColor::parse(&[1, 1, 1, 1]))),
						((0, 1), _Cell::Value(ParsedColor::parse(&[2, 2, 2, 2]))),
						((1, 0), _Cell::Value(ParsedColor::parse(&[3, 3, 3, 3]))),
						((1, 1), _Cell::Value(ParsedColor::parse(&[4, 4, 4, 4]))),
					]),
				},
				extra: (),
			},
			map
		);
		Ok(())
	}

	#[test]
	fn default_cell_when_source_cell_missing() -> Result<(), MapSizeError> {
		let image = _Image(vec![
			(uvec3(0, 0, 0), &[1, 1, 1, 1]),
			(uvec3(0, 1, 0), &[2, 2, 2, 2]),
			(uvec3(1, 1, 0), &[4, 4, 4, 4]),
		]);
		let map = MapCells::<_Cell>::try_parse(&image, &_Lookup)?;

		assert_eq!(
			MapCells {
				definition: CellGridDefinition {
					size: CellGridSize {
						x: new_valid!(CellCount, 2),
						z: new_valid!(CellCount, 2),
					},
					cells: CellGrid::from([
						((0, 0), _Cell::Value(ParsedColor::parse(&[1, 1, 1, 1]))),
						((0, 1), _Cell::Value(ParsedColor::parse(&[2, 2, 2, 2]))),
						((1, 1), _Cell::Value(ParsedColor::parse(&[4, 4, 4, 4]))),
					]),
				},
				extra: (),
			},
			map
		);
		Ok(())
	}

	#[test]
	fn process_3x3() -> Result<(), MapSizeError> {
		let image = _Image(vec![
			(uvec3(0, 0, 0), &[0, 0, 0, 0]),
			(uvec3(0, 1, 0), &[0, 1, 0, 0]),
			(uvec3(0, 2, 0), &[0, 2, 0, 0]),
			(uvec3(1, 0, 0), &[1, 0, 0, 0]),
			(uvec3(1, 1, 0), &[1, 1, 0, 0]),
			(uvec3(1, 2, 0), &[1, 2, 0, 0]),
			(uvec3(2, 0, 0), &[2, 0, 0, 0]),
			(uvec3(2, 1, 0), &[2, 1, 0, 0]),
			(uvec3(2, 2, 0), &[2, 2, 0, 0]),
		]);
		let map = MapCells::<_Cell>::try_parse(&image, &_Lookup)?;

		assert_eq!(
			MapCells {
				definition: CellGridDefinition {
					size: CellGridSize {
						x: new_valid!(CellCount, 3),
						z: new_valid!(CellCount, 3),
					},
					cells: CellGrid::from([
						((0, 0), _Cell::Value(ParsedColor::parse(&[0, 0, 0, 0]))),
						((0, 1), _Cell::Value(ParsedColor::parse(&[0, 1, 0, 0]))),
						((0, 2), _Cell::Value(ParsedColor::parse(&[0, 2, 0, 0]))),
						((1, 0), _Cell::Value(ParsedColor::parse(&[1, 0, 0, 0]))),
						((1, 1), _Cell::Value(ParsedColor::parse(&[1, 1, 0, 0]))),
						((1, 2), _Cell::Value(ParsedColor::parse(&[1, 2, 0, 0]))),
						((2, 0), _Cell::Value(ParsedColor::parse(&[2, 0, 0, 0]))),
						((2, 1), _Cell::Value(ParsedColor::parse(&[2, 1, 0, 0]))),
						((2, 2), _Cell::Value(ParsedColor::parse(&[2, 2, 0, 0]))),
					]),
				},
				extra: (),
			},
			map
		);
		Ok(())
	}

	#[test]
	fn half_offset_from_1_by_1() {
		let definition = CellGridDefinition {
			size: CellGridSize {
				x: new_valid!(CellCount, 1),
				z: new_valid!(CellCount, 1),
			},
			cells: CellGrid::from([]),
		};

		let half_offset_grid = CellGrid::<HalfOffsetCell<_Cell>>::from(&definition);

		assert_eq!(CellGrid::from([]), half_offset_grid);
	}

	#[test]
	fn half_offset_from_2_by_2() {
		let definition = CellGridDefinition {
			size: CellGridSize {
				x: new_valid!(CellCount, 2),
				z: new_valid!(CellCount, 2),
			},
			cells: CellGrid::from([
				((0, 0), _Cell::Value(ParsedColor::parse(&[1, 1, 1, 1]))),
				((0, 1), _Cell::Value(ParsedColor::parse(&[2, 2, 2, 2]))),
				((1, 0), _Cell::Value(ParsedColor::parse(&[3, 3, 3, 3]))),
				((1, 1), _Cell::Value(ParsedColor::parse(&[4, 4, 4, 4]))),
			]),
		};

		let half_offset_grid = CellGrid::<HalfOffsetCell<_Cell>>::from(&definition);

		assert_eq!(
			CellGrid::from([(
				(0, 0),
				HalfOffsetCell::from([
					(
						Direction::Z,
						_Cell::Value(ParsedColor::parse(&[1, 1, 1, 1]))
					),
					(
						Direction::X,
						_Cell::Value(ParsedColor::parse(&[2, 2, 2, 2]))
					),
					(
						Direction::NegX,
						_Cell::Value(ParsedColor::parse(&[3, 3, 3, 3]))
					),
					(
						Direction::NegZ,
						_Cell::Value(ParsedColor::parse(&[4, 4, 4, 4]))
					),
				])
			)]),
			half_offset_grid
		);
	}

	#[test]
	fn half_offset_from_2_by_2_when_cells_missing() {
		let definition = CellGridDefinition {
			size: CellGridSize {
				x: new_valid!(CellCount, 2),
				z: new_valid!(CellCount, 2),
			},
			cells: CellGrid::from([
				((0, 0), _Cell::Value(ParsedColor::parse(&[1, 1, 1, 1]))),
				((0, 1), _Cell::Value(ParsedColor::parse(&[2, 2, 2, 2]))),
				((1, 1), _Cell::Value(ParsedColor::parse(&[4, 4, 4, 4]))),
			]),
		};

		let half_offset_grid = CellGrid::<HalfOffsetCell<_Cell>>::from(&definition);

		assert_eq!(
			CellGrid::from([(
				(0, 0),
				HalfOffsetCell::from([
					(
						Direction::Z,
						_Cell::Value(ParsedColor::parse(&[1, 1, 1, 1]))
					),
					(
						Direction::X,
						_Cell::Value(ParsedColor::parse(&[2, 2, 2, 2]))
					),
					(Direction::NegX, _Cell::Default),
					(
						Direction::NegZ,
						_Cell::Value(ParsedColor::parse(&[4, 4, 4, 4]))
					),
				])
			)]),
			half_offset_grid
		);
	}

	#[test]
	fn half_offset_from_3_by_3() {
		let definition = CellGridDefinition {
			size: CellGridSize {
				x: new_valid!(CellCount, 3),
				z: new_valid!(CellCount, 3),
			},
			cells: CellGrid::from([
				((0, 0), _Cell::Value(ParsedColor::parse(&[0, 0, 0, 0]))),
				((0, 1), _Cell::Value(ParsedColor::parse(&[0, 1, 0, 0]))),
				((0, 2), _Cell::Value(ParsedColor::parse(&[0, 2, 0, 0]))),
				((1, 0), _Cell::Value(ParsedColor::parse(&[1, 0, 0, 0]))),
				((1, 1), _Cell::Value(ParsedColor::parse(&[1, 1, 0, 0]))),
				((1, 2), _Cell::Value(ParsedColor::parse(&[1, 2, 0, 0]))),
				((2, 0), _Cell::Value(ParsedColor::parse(&[2, 0, 0, 0]))),
				((2, 1), _Cell::Value(ParsedColor::parse(&[2, 1, 0, 0]))),
				((2, 2), _Cell::Value(ParsedColor::parse(&[2, 2, 0, 0]))),
			]),
		};

		let half_offset_grid = CellGrid::<HalfOffsetCell<_Cell>>::from(&definition);

		assert_eq!(
			CellGrid::from([
				(
					(0, 0),
					HalfOffsetCell::from([
						(
							Direction::Z,
							_Cell::Value(ParsedColor::parse(&[0, 0, 0, 0]))
						),
						(
							Direction::X,
							_Cell::Value(ParsedColor::parse(&[0, 1, 0, 0]))
						),
						(
							Direction::NegX,
							_Cell::Value(ParsedColor::parse(&[1, 0, 0, 0]))
						),
						(
							Direction::NegZ,
							_Cell::Value(ParsedColor::parse(&[1, 1, 0, 0]))
						),
					])
				),
				(
					(0, 1),
					HalfOffsetCell::from([
						(
							Direction::Z,
							_Cell::Value(ParsedColor::parse(&[0, 1, 0, 0]))
						),
						(
							Direction::X,
							_Cell::Value(ParsedColor::parse(&[0, 2, 0, 0]))
						),
						(
							Direction::NegX,
							_Cell::Value(ParsedColor::parse(&[1, 1, 0, 0]))
						),
						(
							Direction::NegZ,
							_Cell::Value(ParsedColor::parse(&[1, 2, 0, 0]))
						),
					])
				),
				(
					(1, 0),
					HalfOffsetCell::from([
						(
							Direction::Z,
							_Cell::Value(ParsedColor::parse(&[1, 0, 0, 0]))
						),
						(
							Direction::X,
							_Cell::Value(ParsedColor::parse(&[1, 1, 0, 0]))
						),
						(
							Direction::NegX,
							_Cell::Value(ParsedColor::parse(&[2, 0, 0, 0]))
						),
						(
							Direction::NegZ,
							_Cell::Value(ParsedColor::parse(&[2, 1, 0, 0]))
						),
					])
				),
				(
					(1, 1),
					HalfOffsetCell::from([
						(
							Direction::Z,
							_Cell::Value(ParsedColor::parse(&[1, 1, 0, 0]))
						),
						(
							Direction::X,
							_Cell::Value(ParsedColor::parse(&[1, 2, 0, 0]))
						),
						(
							Direction::NegX,
							_Cell::Value(ParsedColor::parse(&[2, 1, 0, 0]))
						),
						(
							Direction::NegZ,
							_Cell::Value(ParsedColor::parse(&[2, 2, 0, 0]))
						),
					])
				),
			]),
			half_offset_grid
		);
	}
}

#[cfg(test)]
mod test_direction {
	use super::*;
	use test_case::test_case;

	#[test_case(Direction::NegZ, Dir3::NEG_Z; "neg z")]
	#[test_case(Direction::NegX, Dir3::NEG_X; "neg x")]
	#[test_case(Direction::Z, Dir3::Z; "z")]
	#[test_case(Direction::X, Dir3::X; "x")]
	fn dir3_from_direction(value: Direction, result: Dir3) {
		assert_eq!(result, Dir3::from(value));
	}

	#[test_case(Direction::NegZ, Direction::NegX, 1; "neg z once")]
	#[test_case(Direction::NegX, Direction::Z, 1; "neg x once")]
	#[test_case(Direction::Z, Direction::X, 1; "z once")]
	#[test_case(Direction::X, Direction::NegZ, 1; "x once")]
	#[test_case(Direction::NegZ, Direction::NegZ, 0; "neg z zero")]
	#[test_case(Direction::NegZ, Direction::Z, 2; "neg z twice")]
	#[test_case(Direction::NegZ, Direction::X, 3; "neg z thrice")]
	fn rotate_right(value: Direction, result: Direction, times: u8) {
		assert_eq!(result, value.rotated_right(times));
	}

	#[test_case(Direction::NegZ, Direction::X; "neg z")]
	#[test_case(Direction::NegX, Direction::NegZ; "neg x")]
	#[test_case(Direction::Z, Direction::NegX; "z")]
	#[test_case(Direction::X, Direction::Z; "x")]
	fn rotate_left(value: Direction, result: Direction) {
		assert_eq!(result, value.rotated_left());
	}
}
