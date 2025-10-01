use crate::{
	resources::map::color_lookup::{MapColorLookup, MapColorLookupImage},
	traits::pixels::PixelBytes,
};
use bevy::prelude::*;
use common::{
	errors::{ErrorData, Level, Unreachable},
	traits::thread_safe::ThreadSafe,
};
use std::{any::type_name, fmt::Display, marker::PhantomData};

impl<TCell> MapColorLookup<TCell>
where
	TCell: ThreadSafe,
{
	pub(crate) fn parse_images(
		commands: Commands,
		lookup: Option<Res<MapColorLookupImage<TCell>>>,
		images: Res<Assets<Image>>,
	) -> Result<(), ParseImageError<TCell>> {
		parse_images(commands, lookup, images)
	}
}

fn parse_images<TCell, TImage>(
	mut commands: Commands,
	lookup: Option<Res<MapColorLookupImage<TCell, TImage>>>,
	images: Res<Assets<TImage>>,
) -> Result<(), ParseImageError<TCell>>
where
	TCell: ThreadSafe,
	TImage: Asset + PixelBytes,
{
	let Some(lookup) = lookup else {
		return Err(ParseImageError::NoLookup);
	};
	let Some(image) = images.get(&lookup.floor) else {
		return Err(ParseImageError::ImageNotLoaded);
	};

	match image.pixel_bytes(UVec3::ZERO) {
		Some([r, g, b, a]) => {
			commands.insert_resource(MapColorLookup::<TCell>::new(Color::srgba_u8(
				*r, *g, *b, *a,
			)));
			Ok(())
		}
		Some(pixel) => Err(ParseImageError::PixelWrongFormat(pixel.to_vec())),
		None => Err(ParseImageError::NoPixels),
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum ParseImageError<TCell> {
	NoLookup,
	ImageNotLoaded,
	NoPixels,
	PixelWrongFormat(Vec<u8>),
	_P(PhantomData<TCell>, Unreachable),
}

impl<TCell> Display for ParseImageError<TCell> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let cell_name = type_name::<TCell>();
		match self {
			ParseImageError::NoLookup => {
				write!(f, "no `ColorLookupImage` loaded for cell type: {cell_name}",)
			}
			ParseImageError::ImageNotLoaded => {
				write!(f, "no floor lookup image loaded for cell type: {cell_name}",)
			}
			ParseImageError::NoPixels => {
				write!(f, "floor lookup image empty for cell type: {cell_name}",)
			}
			ParseImageError::PixelWrongFormat(items) => write!(
				f,
				"{items:?}: pixel format misaligned for floor lookup image with cell type: {cell_name}"
			),
			ParseImageError::_P(..) => unreachable!(),
		}
	}
}

impl<TCell> ErrorData for ParseImageError<TCell> {
	type TContext = Self;

	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> String {
		"Failed to parse image".to_owned()
	}

	fn context(&self) -> &Self::TContext {
		self
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
		lookup_images: Option<MapColorLookupImage<_Cell, _Image>>,
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
			Some(MapColorLookupImage::new(handle.clone())),
			[(handle, image)],
		);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(
			Some(&MapColorLookup::new(Color::srgba_u8(123, 124, 125, 126))),
			app.world().get_resource::<MapColorLookup<_Cell>>(),
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
			Some(MapColorLookupImage::new(handle.clone())),
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

		assert_eq!(Err(ParseImageError::NoLookup), result);
		Ok(())
	}

	#[test]
	fn no_image_error() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let mut app = setup(Some(MapColorLookupImage::new(handle)), []);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		let result = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(Err(ParseImageError::ImageNotLoaded), result);
		Ok(())
	}

	#[test]
	fn no_pixels_error() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(None);
		});
		let mut app = setup(
			Some(MapColorLookupImage::new(handle.clone())),
			[(handle, image)],
		);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		let result = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(Err(ParseImageError::NoPixels), result);
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
			Some(MapColorLookupImage::new(handle.clone())),
			[(handle, image)],
		);

		_ = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		let result = app
			.world_mut()
			.run_system_once(parse_images::<_Cell, _Image>)?;

		assert_eq!(
			Err(ParseImageError::PixelWrongFormat(vec![123, 124, 125])),
			result
		);
		Ok(())
	}
}
