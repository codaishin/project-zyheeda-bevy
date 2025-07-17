use crate::{
	components::map::{cells::MapCells, image::MapImage},
	resources::map::color_lookup::MapColorLookup,
	traits::parse_map_image::ParseMapImage,
};
use bevy::prelude::*;
use common::traits::{
	thread_safe::ThreadSafe,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};

pub(crate) type OkOrParseError<TCells, TCell, TImage = Image> =
	Result<(), Vec<<TCells as ParseMapImage<TImage, TCell>>::TParseError>>;

impl<TCell> MapImage<TCell>
where
	TCell: ThreadSafe,
	MapCells<TCell>: ParseMapImage<Image, TCell>,
{
	pub(crate) fn insert_map_cells(
		commands: Commands,
		handles: Query<(Entity, &Self)>,
		images: Res<Assets<Image>>,
		lookup: Res<MapColorLookup<TCell>>,
	) -> OkOrParseError<MapCells<TCell>, TCell> {
		insert_map_cells::<MapCells<TCell>, TCell, Image>(commands, handles, images, lookup)
	}
}

fn insert_map_cells<TCells, TCell, TImage>(
	mut commands: Commands,
	handles: Query<(Entity, &MapImage<TCell, TImage>)>,
	images: Res<Assets<TImage>>,
	lookup: Res<MapColorLookup<TCell>>,
) -> OkOrParseError<TCells, TCell, TImage>
where
	TCells: ParseMapImage<TImage, TCell> + Component,
	TCell: ThreadSafe,
	TImage: Asset,
{
	let mut errors = vec![];
	for (entity, MapImage { image, .. }) in &handles {
		let Some(image) = images.get(image) else {
			continue;
		};

		let cells = match TCells::try_parse(image, lookup.as_ref()) {
			Ok(cells) => cells,
			Err(error) => {
				errors.push(error);
				continue;
			}
		};

		commands.try_insert_on(entity, cells);
		commands.try_remove_from::<MapImage<TCell, TImage>>(entity);
	}

	if !errors.is_empty() {
		return Err(errors);
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::errors::Unreachable;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath, Debug, PartialEq, Clone, Copy)]
	struct _Image;

	#[derive(Debug, PartialEq)]
	struct _Cell;

	#[derive(Component, Debug, PartialEq)]
	struct _Cells {
		image: _Image,
		lookup: MapColorLookup<_Cell>,
	}

	impl ParseMapImage<_Image, _Cell> for _Cells {
		type TParseError = Unreachable;

		fn try_parse(image: &_Image, lookup: &MapColorLookup<_Cell>) -> Result<Self, Unreachable> {
			Ok(Self {
				image: *image,
				lookup: *lookup,
			})
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _CellsFail;

	impl ParseMapImage<_Image, _Cell> for _CellsFail {
		type TParseError = _Error;

		fn try_parse(_: &_Image, _: &MapColorLookup<_Cell>) -> Result<Self, _Error> {
			Err(_Error)
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Error;

	fn setup<const N: usize>(
		images: [(&Handle<_Image>, _Image); N],
		lookup: MapColorLookup<_Cell>,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, image) in images {
			assets.insert(id, image);
		}
		app.insert_resource(assets);
		app.insert_resource(lookup);

		app
	}

	#[test]
	fn parse_from_image() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let images = [(&handle, _Image)];
		let lookup = MapColorLookup::<_Cell>::new(Color::srgb_u8(1, 2, 3));
		let mut app = setup(images, lookup);
		let entity = app
			.world_mut()
			.spawn(MapImage::<_Cell, _Image>::from(handle))
			.id();

		let result = app
			.world_mut()
			.run_system_once(insert_map_cells::<_Cells, _Cell, _Image>)?;

		assert_eq!(
			(
				Some(&_Cells {
					image: _Image,
					lookup
				}),
				Ok(())
			),
			(app.world().entity(entity).get::<_Cells>(), result),
		);
		Ok(())
	}

	#[test]
	fn remove_image() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let images = [(&handle, _Image)];
		let lookup = MapColorLookup::<_Cell>::new(Color::srgb_u8(1, 2, 3));
		let mut app = setup(images, lookup);
		let entity = app
			.world_mut()
			.spawn(MapImage::<_Cell, _Image>::from(handle))
			.id();

		_ = app
			.world_mut()
			.run_system_once(insert_map_cells::<_Cells, _Cell, _Image>)?;

		assert!(
			!app.world()
				.entity(entity)
				.contains::<MapImage<_Cell, _Image>>(),
		);
		Ok(())
	}

	#[test]
	fn return_error() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let images = [(&handle, _Image)];
		let lookup = MapColorLookup::<_Cell>::new(Color::srgb_u8(1, 2, 3));
		let mut app = setup(images, lookup);
		app.world_mut()
			.spawn(MapImage::<_Cell, _Image>::from(handle));

		let result = app
			.world_mut()
			.run_system_once(insert_map_cells::<_CellsFail, _Cell, _Image>)?;

		assert_eq!(Err(vec![_Error]), result);
		Ok(())
	}

	#[test]
	fn do_not_remove_image_on_error() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let images = [(&handle, _Image)];
		let lookup = MapColorLookup::<_Cell>::new(Color::srgb_u8(1, 2, 3));
		let mut app = setup(images, lookup);
		let entity = app
			.world_mut()
			.spawn(MapImage::<_Cell, _Image>::from(handle))
			.id();

		_ = app
			.world_mut()
			.run_system_once(insert_map_cells::<_CellsFail, _Cell, _Image>)?;

		assert!(
			app.world()
				.entity(entity)
				.contains::<MapImage<_Cell, _Image>>()
		);
		Ok(())
	}
}
