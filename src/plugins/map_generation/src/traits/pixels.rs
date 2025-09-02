mod image;

use bevy::prelude::*;

pub(crate) type Bytes<'a> = &'a [u8];

pub(crate) trait PixelBytes {
	fn pixel_bytes(&self, coords: UVec3) -> Option<Bytes<'_>>;
}

pub(crate) trait PixelBytesIterator {
	type TIter<'a>: Iterator<Item = (UVec3, Bytes<'a>)>
	where
		Self: 'a;

	fn iter_pixel_bytes(&self, layer: Layer) -> Self::TIter<'_>;
}

#[derive(Debug, PartialEq)]
pub(crate) struct Layer(pub(crate) u32);

impl<TImage> PixelBytesIterator for TImage
where
	TImage: PixelBytes,
{
	type TIter<'a>
		= Iter<'a, Self>
	where
		Self: 'a;

	fn iter_pixel_bytes(&self, Layer(z): Layer) -> Self::TIter<'_> {
		Iter {
			coords: UVec3 { x: 0, y: 0, z },
			image: self,
		}
	}
}

pub(crate) struct Iter<'a, TImage>
where
	TImage: PixelBytes,
{
	coords: UVec3,
	image: &'a TImage,
}

impl<'a, TImage> Iter<'a, TImage>
where
	TImage: PixelBytes,
{
	fn get_current_in_line(&mut self) -> Option<(UVec3, Bytes<'a>)> {
		let current = self.get_current_pixel()?;
		self.move_to_next_pixel();
		Some(current)
	}

	fn get_first_in_next_line(&mut self) -> Option<(UVec3, Bytes<'a>)> {
		self.move_to_next_line();
		self.get_current_in_line()
	}

	fn get_current_pixel(&self) -> Option<(UVec3, Bytes<'a>)> {
		Some((self.coords, self.image.pixel_bytes(self.coords)?))
	}

	fn move_to_next_pixel(&mut self) {
		self.coords.x += 1;
	}

	fn move_to_next_line(&mut self) {
		self.coords.x = 0;
		self.coords.y += 1;
	}
}

impl<'a, TImage> Iterator for Iter<'a, TImage>
where
	TImage: PixelBytes,
{
	type Item = (UVec3, Bytes<'a>);

	fn next(&mut self) -> Option<Self::Item> {
		self.get_current_in_line()
			.or_else(|| self.get_first_in_next_line())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use macros::simple_mock;
	use mockall::predicate::eq;
	use testing::Mock;

	simple_mock! {
		_Image {}
		impl PixelBytes for _Image {
			fn pixel_bytes<'a>(&'a self, coords: UVec3) -> Option<&'a [u8]>;
		}
	}

	#[test]
	fn get_0_0() {
		const BYTES: &[u8] = &[1, 2, 3, 4];
		let image = Mock_Image::new_mock(|mock| {
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(0, 0, 0)))
				.return_const(BYTES);
			mock.expect_pixel_bytes().return_const(None);
		});

		let pixels = image.iter_pixel_bytes(Layer(0));

		assert_eq!(vec![(uvec3(0, 0, 0), BYTES)], pixels.collect::<Vec<_>>());
	}

	#[test]
	fn get_one_line_of_pixels() {
		const LINE: [&[u8]; 3] = [&[1, 1, 1, 1], &[2, 2, 2, 2], &[3, 3, 3, 3]];
		let image = Mock_Image::new_mock(|mock| {
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(0, 0, 0)))
				.return_const(LINE[0]);
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(1, 0, 0)))
				.return_const(LINE[1]);
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(2, 0, 0)))
				.return_const(LINE[2]);
			mock.expect_pixel_bytes().return_const(None);
		});

		let pixels = image.iter_pixel_bytes(Layer(0));

		assert_eq!(
			vec![
				(uvec3(0, 0, 0), LINE[0]),
				(uvec3(1, 0, 0), LINE[1]),
				(uvec3(2, 0, 0), LINE[2])
			],
			pixels.collect::<Vec<_>>(),
		);
	}

	#[test]
	fn get_one_column_of_pixels() {
		const COLUMN: [&[u8]; 3] = [&[1, 1, 1, 1], &[2, 2, 2, 2], &[3, 3, 3, 3]];
		let image = Mock_Image::new_mock(|mock| {
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(0, 0, 0)))
				.return_const(COLUMN[0]);
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(0, 1, 0)))
				.return_const(COLUMN[1]);
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(0, 2, 0)))
				.return_const(COLUMN[2]);
			mock.expect_pixel_bytes().return_const(None);
		});

		let pixels = image.iter_pixel_bytes(Layer(0));

		assert_eq!(
			vec![
				(uvec3(0, 0, 0), COLUMN[0]),
				(uvec3(0, 1, 0), COLUMN[1]),
				(uvec3(0, 2, 0), COLUMN[2])
			],
			pixels.collect::<Vec<_>>(),
		);
	}

	#[test]
	fn get_lines_and_columns() {
		const TEXTURE: [[&[u8]; 2]; 2] = [
			[&[1, 1, 1, 1], &[2, 2, 2, 2]],
			[&[3, 3, 3, 3], &[4, 4, 4, 4]],
		];
		let image = Mock_Image::new_mock(|mock| {
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(0, 0, 0)))
				.return_const(TEXTURE[0][0]);
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(0, 1, 0)))
				.return_const(TEXTURE[1][0]);
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(1, 0, 0)))
				.return_const(TEXTURE[0][1]);
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(1, 1, 0)))
				.return_const(TEXTURE[1][1]);
			mock.expect_pixel_bytes().return_const(None);
		});

		let pixels = image.iter_pixel_bytes(Layer(0));

		assert_eq!(
			vec![
				(uvec3(0, 0, 0), TEXTURE[0][0]),
				(uvec3(1, 0, 0), TEXTURE[0][1]),
				(uvec3(0, 1, 0), TEXTURE[1][0]),
				(uvec3(1, 1, 0), TEXTURE[1][1]),
			],
			pixels.collect::<Vec<_>>(),
		);
	}

	#[test]
	fn get_0_0_on_deeper_layer() {
		const BYTES: &[u8] = &[1, 2, 3, 4];
		let image = Mock_Image::new_mock(|mock| {
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(uvec3(0, 0, 1)))
				.return_const(BYTES);
			mock.expect_pixel_bytes().return_const(None);
		});

		let pixels = image.iter_pixel_bytes(Layer(1));

		assert_eq!(vec![(uvec3(0, 0, 1), BYTES)], pixels.collect::<Vec<_>>());
	}
}
