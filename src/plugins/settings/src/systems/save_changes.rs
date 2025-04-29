use crate::resources::asset_writer::{AssetWriter, WriteAsset, WriteError};
use bevy::prelude::*;
use common::traits::load_asset::Path;
use serde::Serialize;

impl<T> SaveChanges for T where T: Resource + Clone {}

pub(crate) trait SaveChanges: Resource + Clone + Sized {
	fn save_changes<TDto>(
		path: Path,
	) -> impl Fn(Res<Self>, Res<AssetWriter>) -> Result<(), WriteError>
	where
		TDto: Serialize + From<Self> + 'static,
	{
		save_changes::<Self, TDto, AssetWriter>(path)
	}
}

fn save_changes<TAsset, TDto, TWriter>(
	path: Path,
) -> impl Fn(Res<TAsset>, Res<TWriter>) -> Result<(), TWriter::TError>
where
	TAsset: Resource + Clone,
	TDto: Serialize + From<TAsset> + 'static,
	TWriter: WriteAsset + Resource,
{
	move |resource, writer| {
		if !resource.is_changed() {
			return Ok(());
		}

		let dto = TDto::from(resource.clone());
		writer.write(dto, path.clone())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Debug, PartialEq, Serialize, Clone)]
	struct _Resource;

	#[derive(Asset, TypePath, Debug, PartialEq, Serialize)]
	struct _ResourceDto(_Resource);

	impl From<_Resource> for _ResourceDto {
		fn from(resource: _Resource) -> Self {
			Self(resource)
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Writer {
		mock: Mock_Writer,
	}

	#[automock]
	impl WriteAsset for _Writer {
		type TError = _Error;

		fn write<TAsset>(&self, asset: TAsset, path: Path) -> Result<(), _Error>
		where
			TAsset: Serialize + 'static,
		{
			self.mock.write(asset, path)
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	pub struct _Error;

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), _Error>);

	fn setup(writer: _Writer, asset: _Resource, path: Path) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			save_changes::<_Resource, _ResourceDto, _Writer>(path).pipe(
				|In(r), mut commands: Commands| {
					commands.insert_resource(_Result(r));
				},
			),
		);
		app.insert_resource(writer);
		app.insert_resource(asset);

		app
	}

	#[test]
	fn call_writer() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write()
				.times(1)
				.with(eq(_ResourceDto(_Resource)), eq(Path::from("my/path")))
				.return_const(Ok(()));
		});
		let mut app = setup(writer, _Resource, Path::from("my/path"));

		app.update();
	}

	#[test]
	fn return_result() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write::<_ResourceDto>()
				.return_const(Err(_Error));
		});
		let mut app = setup(writer, _Resource, Path::from("my/path"));

		app.update();

		assert_eq!(&_Result(Err(_Error)), app.world().resource::<_Result>());
	}

	#[test]
	fn call_writer_only_once() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write::<_ResourceDto>()
				.times(1)
				.return_const(Ok(()));
		});
		let mut app = setup(writer, _Resource, Path::from("my/path"));

		app.update();
		app.update();
	}

	#[test]
	fn call_writer_again_when_resource_changed() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write::<_ResourceDto>()
				.times(2)
				.return_const(Ok(()));
		});
		let mut app = setup(writer, _Resource, Path::from("my/path"));

		app.update();
		app.world_mut()
			.get_resource_mut::<_Resource>()
			.as_deref_mut();
		app.update();
	}
}
