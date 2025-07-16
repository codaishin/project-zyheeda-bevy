use crate::{
	map_cells::{MapSizeError, half_offset_cell::HalfOffsetCell, parsed_color::ParsedColor},
	resources::color_lookup::ColorLookup,
	traits::{
		parse_map_image::ParseMapImage,
		pixels::{Layer, PixelBytesIterator},
	},
};
use bevy::prelude::*;
use common::{errors::Unreachable, traits::thread_safe::ThreadSafe};
use std::collections::HashMap;

pub(crate) type CellGrid<TCell> = HashMap<(usize, usize), TCell>;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct MapCells<TCell> {
	pub(crate) size: Size,
	pub(crate) cells: CellGrid<TCell>,
	pub(crate) half_offset_cells: CellGrid<HalfOffsetCell<TCell>>,
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct Size {
	pub(crate) x: usize,
	pub(crate) z: usize,
}

impl<TCell> Default for MapCells<TCell> {
	fn default() -> Self {
		Self {
			size: Size::default(),
			cells: CellGrid::default(),
			half_offset_cells: CellGrid::default(),
		}
	}
}

impl<TCell, TImage> ParseMapImage<TImage, TCell> for MapCells<TCell>
where
	TImage: PixelBytesIterator,
	for<'a> TCell:
		ParseMapImage<ParsedColor, TCell, TParseError = Unreachable> + Clone + ThreadSafe + Default,
{
	type TParseError = MapSizeError;

	fn try_parse(image: &TImage, lookup: &ColorLookup<TCell>) -> Result<Self, MapSizeError> {
		let mut max = Size { x: 0, z: 0 };
		let mut cells = CellGrid::default();
		let mut half_offset_cells = CellGrid::default();

		for (UVec3 { x, y, .. }, bytes) in image.iter_pixel_bytes(Layer(0)) {
			let x = x as usize;
			let z = y as usize;
			max.x = usize::max(x, max.x);
			max.z = usize::max(z, max.z);

			let Ok(cell) = TCell::try_parse(&ParsedColor::parse(bytes), lookup);
			cells.insert((x, z), cell);
		}

		if cells.is_empty() {
			return Err(MapSizeError::Empty);
		}

		if max.x == 0 || max.z == 0 {
			return Err(MapSizeError::Sizes {
				x: max.x + 1,
				z: max.z + 1,
			});
		}

		let size = Size {
			x: max.x + 1,
			z: max.z + 1,
		};
		for x in 1..size.x {
			for z in 1..size.z {
				let directions = HalfOffsetCell::directions(x, z).map(|(x, z, dir)| {
					let cell = cells.get(&(x, z)).cloned().unwrap_or_default();
					(dir, cell)
				});
				half_offset_cells.insert((x - 1, z - 1), HalfOffsetCell::from(directions));
			}
		}

		Ok(Self {
			size,
			cells,
			half_offset_cells,
		})
	}
}

#[cfg(test)]
mod test_from_image {
	use super::*;
	use crate::{
		map_cells::Direction,
		traits::pixels::{Bytes, Layer},
	};
	use mockall::{mock, predicate::eq};
	use std::vec::IntoIter;
	use testing::{Mock, simple_init};

	#[derive(TypePath, Debug, PartialEq, Clone, Default)]
	enum _Cell {
		#[default]
		Default,
		Value((ParsedColor, ColorLookup<Self>)),
	}

	impl ParseMapImage<ParsedColor, Self> for _Cell {
		type TParseError = Unreachable;

		fn try_parse(
			parsed_color: &ParsedColor,
			lookup: &ColorLookup<Self>,
		) -> Result<Self, Self::TParseError> {
			Ok(_Cell::Value((*parsed_color, *lookup)))
		}
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
		let lookup = ColorLookup::new(Color::srgba_u8(0, 0, 0, 0));

		let map = MapCells::<_Cell>::try_parse(&image, &lookup);

		assert_eq!(Err(MapSizeError::Empty), map);
	}

	#[test]
	fn too_small_z() {
		let image = _Image(vec![(uvec3(0, 0, 0), &[]), (uvec3(1, 0, 0), &[])]);
		let lookup = ColorLookup::new(Color::srgba_u8(0, 0, 0, 0));

		let map = MapCells::<_Cell>::try_parse(&image, &lookup);

		assert_eq!(Err(MapSizeError::Sizes { x: 2, z: 1 }), map);
	}

	#[test]
	fn too_small_x() {
		let image = _Image(vec![(uvec3(0, 0, 0), &[]), (uvec3(0, 1, 0), &[])]);
		let lookup = ColorLookup::new(Color::srgba_u8(0, 0, 0, 0));

		let map = MapCells::<_Cell>::try_parse(&image, &lookup);

		assert_eq!(Err(MapSizeError::Sizes { x: 1, z: 2 }), map);
	}

	#[test]
	fn too_small_x_and_z() {
		let image = _Image(vec![(uvec3(0, 0, 0), &[])]);
		let lookup = ColorLookup::new(Color::srgba_u8(0, 0, 0, 0));

		let map = MapCells::<_Cell>::try_parse(&image, &lookup);

		assert_eq!(Err(MapSizeError::Sizes { x: 1, z: 1 }), map);
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
		let lookup = ColorLookup::new(Color::srgba_u8(0, 0, 0, 0));

		_ = MapCells::<_Cell>::try_parse(&image, &lookup)
	}

	#[test]
	fn parse_pixels() -> Result<(), MapSizeError> {
		let image = _Image(vec![
			(uvec3(0, 0, 0), &[1, 1, 1, 1]),
			(uvec3(0, 1, 0), &[2, 2, 2, 2]),
			(uvec3(1, 0, 0), &[3, 3, 3, 3]),
			(uvec3(1, 1, 0), &[4, 4, 4, 4]),
		]);
		let lookup = ColorLookup::new(Color::srgb_u8(1, 2, 3));
		let map = MapCells::<_Cell>::try_parse(&image, &lookup)?;

		assert_eq!(
			MapCells {
				size: Size { x: 2, z: 2 },
				cells: CellGrid::from([
					(
						(0, 0),
						_Cell::Value((ParsedColor::parse(&[1, 1, 1, 1]), lookup))
					),
					(
						(0, 1),
						_Cell::Value((ParsedColor::parse(&[2, 2, 2, 2]), lookup))
					),
					(
						(1, 0),
						_Cell::Value((ParsedColor::parse(&[3, 3, 3, 3]), lookup))
					),
					(
						(1, 1),
						_Cell::Value((ParsedColor::parse(&[4, 4, 4, 4]), lookup))
					),
				]),
				half_offset_cells: CellGrid::from([(
					(0, 0),
					HalfOffsetCell::from([
						(
							Direction::Z,
							_Cell::Value((ParsedColor::parse(&[1, 1, 1, 1]), lookup))
						),
						(
							Direction::X,
							_Cell::Value((ParsedColor::parse(&[2, 2, 2, 2]), lookup))
						),
						(
							Direction::NegX,
							_Cell::Value((ParsedColor::parse(&[3, 3, 3, 3]), lookup))
						),
						(
							Direction::NegZ,
							_Cell::Value((ParsedColor::parse(&[4, 4, 4, 4]), lookup))
						),
					])
				)]),
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
		let lookup = ColorLookup::new(Color::srgb_u8(1, 2, 3));
		let map = MapCells::<_Cell>::try_parse(&image, &lookup)?;

		assert_eq!(
			MapCells {
				size: Size { x: 2, z: 2 },
				cells: CellGrid::from([
					(
						(0, 0),
						_Cell::Value((ParsedColor::parse(&[1, 1, 1, 1]), lookup))
					),
					(
						(0, 1),
						_Cell::Value((ParsedColor::parse(&[2, 2, 2, 2]), lookup))
					),
					(
						(1, 1),
						_Cell::Value((ParsedColor::parse(&[4, 4, 4, 4]), lookup))
					),
				]),
				half_offset_cells: CellGrid::from([(
					(0, 0),
					HalfOffsetCell::from([
						(
							Direction::Z,
							_Cell::Value((ParsedColor::parse(&[1, 1, 1, 1]), lookup))
						),
						(
							Direction::X,
							_Cell::Value((ParsedColor::parse(&[2, 2, 2, 2]), lookup))
						),
						(Direction::NegX, _Cell::Default),
						(
							Direction::NegZ,
							_Cell::Value((ParsedColor::parse(&[4, 4, 4, 4]), lookup))
						),
					])
				)]),
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
		let lookup = ColorLookup::new(Color::srgb_u8(1, 2, 3));
		let map = MapCells::<_Cell>::try_parse(&image, &lookup)?;

		assert_eq!(
			MapCells {
				size: Size { x: 3, z: 3 },
				cells: CellGrid::from([
					(
						(0, 0),
						_Cell::Value((ParsedColor::parse(&[0, 0, 0, 0]), lookup))
					),
					(
						(0, 1),
						_Cell::Value((ParsedColor::parse(&[0, 1, 0, 0]), lookup))
					),
					(
						(0, 2),
						_Cell::Value((ParsedColor::parse(&[0, 2, 0, 0]), lookup))
					),
					(
						(1, 0),
						_Cell::Value((ParsedColor::parse(&[1, 0, 0, 0]), lookup))
					),
					(
						(1, 1),
						_Cell::Value((ParsedColor::parse(&[1, 1, 0, 0]), lookup))
					),
					(
						(1, 2),
						_Cell::Value((ParsedColor::parse(&[1, 2, 0, 0]), lookup))
					),
					(
						(2, 0),
						_Cell::Value((ParsedColor::parse(&[2, 0, 0, 0]), lookup))
					),
					(
						(2, 1),
						_Cell::Value((ParsedColor::parse(&[2, 1, 0, 0]), lookup))
					),
					(
						(2, 2),
						_Cell::Value((ParsedColor::parse(&[2, 2, 0, 0]), lookup))
					),
				]),
				half_offset_cells: CellGrid::from([
					(
						(0, 0),
						HalfOffsetCell::from([
							(
								Direction::Z,
								_Cell::Value((ParsedColor::parse(&[0, 0, 0, 0]), lookup))
							),
							(
								Direction::X,
								_Cell::Value((ParsedColor::parse(&[0, 1, 0, 0]), lookup))
							),
							(
								Direction::NegX,
								_Cell::Value((ParsedColor::parse(&[1, 0, 0, 0]), lookup))
							),
							(
								Direction::NegZ,
								_Cell::Value((ParsedColor::parse(&[1, 1, 0, 0]), lookup))
							),
						])
					),
					(
						(0, 1),
						HalfOffsetCell::from([
							(
								Direction::Z,
								_Cell::Value((ParsedColor::parse(&[0, 1, 0, 0]), lookup))
							),
							(
								Direction::X,
								_Cell::Value((ParsedColor::parse(&[0, 2, 0, 0]), lookup))
							),
							(
								Direction::NegX,
								_Cell::Value((ParsedColor::parse(&[1, 1, 0, 0]), lookup))
							),
							(
								Direction::NegZ,
								_Cell::Value((ParsedColor::parse(&[1, 2, 0, 0]), lookup))
							),
						])
					),
					(
						(1, 0),
						HalfOffsetCell::from([
							(
								Direction::Z,
								_Cell::Value((ParsedColor::parse(&[1, 0, 0, 0]), lookup))
							),
							(
								Direction::X,
								_Cell::Value((ParsedColor::parse(&[1, 1, 0, 0]), lookup))
							),
							(
								Direction::NegX,
								_Cell::Value((ParsedColor::parse(&[2, 0, 0, 0]), lookup))
							),
							(
								Direction::NegZ,
								_Cell::Value((ParsedColor::parse(&[2, 1, 0, 0]), lookup))
							),
						])
					),
					(
						(1, 1),
						HalfOffsetCell::from([
							(
								Direction::Z,
								_Cell::Value((ParsedColor::parse(&[1, 1, 0, 0]), lookup))
							),
							(
								Direction::X,
								_Cell::Value((ParsedColor::parse(&[1, 2, 0, 0]), lookup))
							),
							(
								Direction::NegX,
								_Cell::Value((ParsedColor::parse(&[2, 1, 0, 0]), lookup))
							),
							(
								Direction::NegZ,
								_Cell::Value((ParsedColor::parse(&[2, 2, 0, 0]), lookup))
							),
						])
					),
				]),
			},
			map
		);
		Ok(())
	}
}
