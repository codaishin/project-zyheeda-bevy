use crate::{
	resources::color_lookup::{ColorLookup, ColorLookupImage},
	traits::pixels::PixelBytes,
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level, Unreachable},
	traits::thread_safe::ThreadSafe,
};
use std::{any::type_name, marker::PhantomData};

impl<TCell> ColorLookup<TCell>
where
	TCell: ThreadSafe,
{
	pub(crate) fn parse_images(
		commands: Commands,
		lookup: Option<Res<ColorLookupImage<TCell>>>,
		images: Res<Assets<Image>>,
	) -> Result<(), ParseImagesError<TCell>> {
		parse_images(commands, lookup, images)
	}
}

fn parse_images<TCell, TImage>(
	mut commands: Commands,
	lookup: Option<Res<ColorLookupImage<TCell, TImage>>>,
	images: Res<Assets<TImage>>,
) -> Result<(), ParseImagesError<TCell>>
where
	TCell: ThreadSafe,
	TImage: Asset + PixelBytes,
{
	let Some(lookup) = lookup else {
		return Err(ParseImagesError::NoLookup);
	};
	let Some(image) = images.get(&lookup.floor) else {
		return Err(ParseImagesError::ImageNotLoaded);
	};

	match image.pixel_bytes(UVec3::ZERO) {
		Some([r, g, b, a]) => {
			commands.insert_resource(ColorLookup::<TCell>::new(Color::srgba_u8(*r, *g, *b, *a)));
			Ok(())
		}
		Some(pixel) => Err(ParseImagesError::PixelWrongFormat(pixel.to_vec())),
		None => Err(ParseImagesError::NoPixels),
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum ParseImagesError<TCell> {
	NoLookup,
	ImageNotLoaded,
	NoPixels,
	PixelWrongFormat(Vec<u8>),
	_P(PhantomData<TCell>, Unreachable),
}

impl<TCell> From<ParseImagesError<TCell>> for Error {
	fn from(value: ParseImagesError<TCell>) -> Self {
		let cell_name = type_name::<TCell>();
		match value {
			ParseImagesError::NoLookup => Self::Single {
				msg: format!("no `ColorLookupImage` loaded for cell type: {cell_name}"),
				lvl: Level::Warning,
			},
			ParseImagesError::ImageNotLoaded => Self::Single {
				msg: format!("no floor lookup image loaded for cell type: {cell_name}"),
				lvl: Level::Warning,
			},
			ParseImagesError::NoPixels => Self::Single {
				msg: format!("floor lookup image empty for cell type: {cell_name}"),
				lvl: Level::Error,
			},
			ParseImagesError::PixelWrongFormat(items) => Self::Single {
				msg: format!(
					"{items:?}: pixel format misaligned for floor lookup image with cell type: {cell_name}"
				),
				lvl: Level::Error,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, assert_no_panic, new_handle};

	#[derive(Debug, PartialEq)]
	struct _Cell;

	#[derive(Asset, TypePath, NestedMocks)]
	struct _Image {
		mock: Mock_Image,
	}

	#[automock]
	impl PixelBytes for _Image {
		#[allow(clippy::needless_lifetimes)]
		fn pixel_bytes<'a>(&'a self, coords: UVec3) -> Option<&'a [u8]> {
			self.mock.pixel_bytes(coords)
		}
	}

	fn setup(
		lookup_images: Option<ColorLookupImage<_Cell, _Image>>,
		images: impl IntoIterator<Item = (Handle<_Image>, _Image)>,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		if let Some(lookup_images) = lookup_images {
			app.insert_resource(lookup_images);
		}
		for (handle, image) in images.into_iter() {
			assets.insert(&handle, image);
		}
		app.insert_resource(assets);

		app
	}

	#[test]
	fn parse() -> Result<(), RunSystemError> {
		const COLOR: &[u8] = &[123, 124, 125, 126];
		let handle = new_handle();
		let image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(COLOR);
		});
		let mut app = setup(
			Some(ColorLookupImage::new(handle.clone())),
			[(handle, image)],
		);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(
			Some(&ColorLookup::new(Color::srgba_u8(123, 124, 125, 126))),
			app.world().get_resource::<ColorLookup<_Cell>>(),
		);
		Ok(())
	}

	#[test]
	fn use_first_pixel_on_layer_one() -> Result<(), RunSystemError> {
		const COLOR: &[u8] = &[123, 124, 125, 126];
		let handle = new_handle();
		let image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(UVec3::ZERO))
				.return_const(COLOR);
		});
		let mut app = setup(
			Some(ColorLookupImage::new(handle.clone())),
			[(handle, image)],
		);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;
		Ok(())
	}

	#[test]
	fn no_panic_if_lookup_image_missing() -> Result<(), RunSystemError> {
		const COLOR: &[u8] = &[123, 124, 125, 126];
		let handle = new_handle();
		let image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(COLOR);
		});
		let mut app = setup(None, [(handle, image)]);

		assert_no_panic! {
			_ =app.world_mut()
				.run_system_once(parse_images::<_Cell, _Image>)?
		}
		Ok(())
	}

	#[test]
	fn no_lookup_error() -> Result<(), RunSystemError> {
		const COLOR: &[u8] = &[123, 124, 125, 126];
		let handle = new_handle();
		let image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(COLOR);
		});
		let mut app = setup(None, [(handle, image)]);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		let result = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(Err(ParseImagesError::NoLookup), result);
		Ok(())
	}

	#[test]
	fn no_image_error() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let mut app = setup(Some(ColorLookupImage::new(handle)), []);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		let result = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(Err(ParseImagesError::ImageNotLoaded), result);
		Ok(())
	}

	#[test]
	fn no_pixels_error() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(None);
		});
		let mut app = setup(
			Some(ColorLookupImage::new(handle.clone())),
			[(handle, image)],
		);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		let result = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(Err(ParseImagesError::NoPixels), result);
		Ok(())
	}

	#[test]
	fn pixel_wrong_format_error() -> Result<(), RunSystemError> {
		const COLOR: &[u8] = &[123, 124, 125];
		let handle = new_handle();
		let image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(COLOR);
		});
		let mut app = setup(
			Some(ColorLookupImage::new(handle.clone())),
			[(handle, image)],
		);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		let result = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(
			Err(ParseImagesError::PixelWrongFormat(vec![123, 124, 125])),
			result
		);
		Ok(())
	}
}
