use crate::{
	components::map::{cells::MapCells, image::MapImage},
	traits::parse_map_image::ParseMapImage,
};
use bevy::prelude::*;
use common::traits::{
	thread_safe::ThreadSafe,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};

type MapCellsLookup<TCell> = <MapCells<TCell> as ParseMapImage<Image>>::TLookup;
type MapCellsParseError<TCell> = <MapCells<TCell> as ParseMapImage<Image>>::TParseError;

impl<TCell> MapImage<TCell>
where
	TCell: ThreadSafe,
	MapCells<TCell>: ParseMapImage<Image>,
	MapCellsLookup<TCell>: Resource,
{
	pub(crate) fn insert_map_cells(
		commands: Commands,
		handles: Query<(Entity, &Self)>,
		images: Res<Assets<Image>>,
		lookup: Res<MapCellsLookup<TCell>>,
	) -> Result<(), Vec<MapCellsParseError<TCell>>> {
		insert_map_cells::<MapCells<TCell>, TCell, Image>(commands, handles, images, lookup)
	}
}

fn insert_map_cells<TCells, TCell, TImage>(
	mut commands: Commands,
	handles: Query<(Entity, &MapImage<TCell, TImage>)>,
	images: Res<Assets<TImage>>,
	lookup: Res<TCells::TLookup>,
) -> Result<(), Vec<TCells::TParseError>>
where
	TCells: ParseMapImage<TImage> + Component,
	TCells::TLookup: Resource,
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

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Lookup;

	#[derive(Asset, TypePath, Debug, PartialEq, Clone, Copy)]
	struct _Image;

	#[derive(Debug, PartialEq)]
	struct _Cell;

	#[derive(Component, Debug, PartialEq)]
	struct _Cells;

	impl ParseMapImage<_Image> for _Cells {
		type TParseError = Unreachable;
		type TLookup = _Lookup;

		fn try_parse(_: &_Image, _: &_Lookup) -> Result<Self, Unreachable> {
			Ok(Self)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _CellsFail;

	impl ParseMapImage<_Image> for _CellsFail {
		type TParseError = _Error;
		type TLookup = _Lookup;

		fn try_parse(_: &_Image, _: &_Lookup) -> Result<Self, _Error> {
			Err(_Error)
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Error;

	fn setup<const N: usize>(images: [(&Handle<_Image>, _Image); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, image) in images {
			assets.insert(id, image);
		}
		app.insert_resource(assets);
		app.init_resource::<_Lookup>();

		app
	}

	#[test]
	fn parse_from_image() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let images = [(&handle, _Image)];
		let mut app = setup(images);
		let entity = app
			.world_mut()
			.spawn(MapImage::<_Cell, _Image>::from(handle))
			.id();

		let result = app
			.world_mut()
			.run_system_once(insert_map_cells::<_Cells, _Cell, _Image>)?;

		assert_eq!(
			(Some(&_Cells), Ok(())),
			(app.world().entity(entity).get::<_Cells>(), result),
		);
		Ok(())
	}

	#[test]
	fn remove_image() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let images = [(&handle, _Image)];
		let mut app = setup(images);
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
		let mut app = setup(images);
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
		let mut app = setup(images);
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
