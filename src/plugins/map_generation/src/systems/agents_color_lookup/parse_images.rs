use crate::{
	resources::agents::color_lookup::{AgentsLookup, AgentsLookupImages},
	systems::map_color_lookup::parse_images::ParseImageError,
	traits::pixels::PixelBytes,
};
use bevy::prelude::*;

impl AgentsLookup {
	pub(crate) fn parse_images(
		commands: Commands,
		lookup: Option<Res<AgentsLookupImages>>,
		images: Res<Assets<Image>>,
	) -> Result<(), Vec<ParseImageError<()>>> {
		parse_images(commands, lookup, images)
	}
}

fn parse_images<TImage>(
	mut commands: Commands,
	lookup: Option<Res<AgentsLookupImages<TImage>>>,
	images: Res<Assets<TImage>>,
) -> Result<(), Vec<ParseImageError<()>>>
where
	TImage: Asset + PixelBytes,
{
	let Some(lookup) = lookup else {
		return Err(vec![ParseImageError::NoLookup]);
	};

	match player_and_enemy(images, lookup) {
		[Ok(player), Ok(enemy)] => {
			commands.insert_resource(AgentsLookup { player, enemy });
			Ok(())
		}
		result => Err(result.into_iter().filter_map(|r| r.err()).collect()),
	}
}

fn player_and_enemy<TImage>(
	images: Res<Assets<TImage>>,
	lookup: Res<AgentsLookupImages<TImage>>,
) -> [Result<Color, ParseImageError<()>>; 2]
where
	TImage: Asset + PixelBytes,
{
	[&lookup.player, &lookup.enemy].map(|handle| {
		let Some(image) = images.get(handle) else {
			return Err(ParseImageError::ImageNotLoaded);
		};
		match image.pixel_bytes(UVec3::ZERO) {
			Some([r, g, b, a]) => Ok(Color::srgba_u8(*r, *g, *b, *a)),
			Some(pixel) => Err(ParseImageError::PixelWrongFormat(pixel.to_vec())),
			None => Err(ParseImageError::NoPixels),
		}
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

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
		lookup_images: Option<AgentsLookupImages<_Image>>,
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
	fn parse_player_and_enemy_images() -> Result<(), RunSystemError> {
		const PLAYER_COLOR: &[u8] = &[123, 124, 125, 126];
		const ENEMY_COLOR: &[u8] = &[113, 114, 115, 116];
		let player_handle = new_handle();
		let enemy_handle = new_handle();
		let player_image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(PLAYER_COLOR);
		});
		let enemy_image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(ENEMY_COLOR);
		});
		let mut app = setup(
			Some(AgentsLookupImages {
				player: player_handle.clone(),
				enemy: enemy_handle.clone(),
			}),
			[(player_handle, player_image), (enemy_handle, enemy_image)],
		);

		_ = app.world_mut().run_system_once(parse_images::<_Image>)?;

		assert_eq!(
			Some(&AgentsLookup {
				player: Color::srgba_u8(123, 124, 125, 126),
				enemy: Color::srgba_u8(113, 114, 115, 116),
			}),
			app.world().get_resource::<AgentsLookup>(),
		);
		Ok(())
	}

	#[test]
	fn use_first_pixel_on_layer_one() -> Result<(), RunSystemError> {
		const PLAYER_COLOR: &[u8] = &[123, 124, 125, 126];
		const ENEMY_COLOR: &[u8] = &[113, 114, 115, 116];
		let player_handle = new_handle();
		let enemy_handle = new_handle();
		let player_image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(UVec3::ZERO))
				.return_const(PLAYER_COLOR);
		});
		let enemy_image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes()
				.times(1)
				.with(eq(UVec3::ZERO))
				.return_const(ENEMY_COLOR);
		});
		let mut app = setup(
			Some(AgentsLookupImages {
				player: player_handle.clone(),
				enemy: enemy_handle.clone(),
			}),
			[(player_handle, player_image), (enemy_handle, enemy_image)],
		);

		_ = app.world_mut().run_system_once(parse_images::<_Image>)?;
		Ok(())
	}

	#[test]
	fn no_lookup_error() -> Result<(), RunSystemError> {
		let mut app = setup(None, []);

		let result = app.world_mut().run_system_once(parse_images::<_Image>)?;
		assert_eq!(Err(vec![ParseImageError::NoLookup]), result);
		Ok(())
	}

	#[test]
	fn no_image_errors() -> Result<(), RunSystemError> {
		let mut app = setup(
			Some(AgentsLookupImages {
				player: new_handle(),
				enemy: new_handle(),
			}),
			[],
		);

		let result = app.world_mut().run_system_once(parse_images::<_Image>)?;

		assert_eq!(
			Err(vec![
				ParseImageError::ImageNotLoaded,
				ParseImageError::ImageNotLoaded,
			]),
			result
		);
		Ok(())
	}

	#[test]
	fn no_pixels_errors() -> Result<(), RunSystemError> {
		let player_handle = new_handle();
		let enemy_handle = new_handle();
		let player_image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(None);
		});
		let enemy_image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(None);
		});
		let mut app = setup(
			Some(AgentsLookupImages {
				player: player_handle.clone(),
				enemy: enemy_handle.clone(),
			}),
			[(player_handle, player_image), (enemy_handle, enemy_image)],
		);

		let result = app.world_mut().run_system_once(parse_images::<_Image>)?;

		assert_eq!(
			Err(vec![ParseImageError::NoPixels, ParseImageError::NoPixels]),
			result
		);
		Ok(())
	}

	#[test]
	fn pixel_wrong_format_errors() -> Result<(), RunSystemError> {
		const PLAYER_COLOR: &[u8] = &[123, 124];
		const ENEMY_COLOR: &[u8] = &[113, 114];
		let player_handle = new_handle();
		let enemy_handle = new_handle();
		let player_image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(PLAYER_COLOR);
		});
		let enemy_image = _Image::new().with_mock(|mock| {
			mock.expect_pixel_bytes().return_const(ENEMY_COLOR);
		});
		let mut app = setup(
			Some(AgentsLookupImages {
				player: player_handle.clone(),
				enemy: enemy_handle.clone(),
			}),
			[(player_handle, player_image), (enemy_handle, enemy_image)],
		);

		let result = app.world_mut().run_system_once(parse_images::<_Image>)?;

		assert_eq!(
			Err(vec![
				ParseImageError::PixelWrongFormat(vec![123, 124]),
				ParseImageError::PixelWrongFormat(vec![113, 114]),
			]),
			result
		);
		Ok(())
	}
}
