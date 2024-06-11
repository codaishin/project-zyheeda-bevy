use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
use bevy::{
	asset::Handle,
	ecs::system::In,
	prelude::{Commands, Entity, Query, ResMut, Resource},
	render::texture::Image,
	ui::UiImage,
};
use common::traits::{
	cache::{GetOrLoadAsset, GetOrLoadAssetFactory},
	load_asset::Path,
	try_insert_on::TryInsertOn,
};

pub(crate) fn set_quickbar_icons<TAssets, TStorage, TFactory>(
	icon_paths: In<Vec<(Entity, Option<Path>)>>,
	mut commands: Commands,
	mut panels: Query<&mut QuickbarPanel>,
	assets: ResMut<TAssets>,
	storage: ResMut<TStorage>,
) where
	TAssets: Resource,
	TStorage: Resource,
	TFactory: GetOrLoadAssetFactory<TAssets, Image, TStorage>,
{
	let mut cache = TFactory::create_from(assets, storage);
	for (entity, icon) in icon_paths.0 {
		let Ok(mut panel) = panels.get_mut(entity) else {
			continue;
		};

		let (state, image) = match icon {
			Some(icon) => (PanelState::Filled, cache.get_or_load(icon)),
			None => (PanelState::Empty, Handle::default()),
		};

		panel.state = state;
		commands.try_insert_on(entity, UiImage::new(image));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::PanelState;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		ecs::system::Res,
		prelude::{IntoSystem, ResMut},
		ui::UiImage,
		utils::Uuid,
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::cache::GetOrLoadAsset};
	use mockall::{mock, predicate::eq};
	use skills::items::slot_key::SlotKey;

	#[derive(Resource, Default)]
	struct _Assets;

	#[derive(Resource, Default)]
	struct _Storage;

	#[derive(Resource)]
	struct _IconPaths(Vec<(Entity, Option<Path>)>);

	mock! {
		_Cache {}
		impl GetOrLoadAsset<Image> for _Cache {
			fn get_or_load(&mut self, key: Path) -> Handle<Image>;
		}
	}

	fn get_icon_paths(data: Res<_IconPaths>) -> Vec<(Entity, Option<Path>)> {
		data.0.clone()
	}

	fn setup<TFactory: GetOrLoadAssetFactory<_Assets, Image, _Storage> + 'static>() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Assets>();
		app.init_resource::<_Storage>();
		app.add_systems(
			Update,
			get_icon_paths.pipe(set_quickbar_icons::<_Assets, _Storage, TFactory>),
		);

		app
	}

	fn arbitrary_key() -> SlotKey {
		SlotKey::default()
	}

	#[test]
	fn add_icon_image() {
		const HANDLE: Handle<Image> = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(0xe5db01e4_9a32_43d1_b048_d690d646adde),
		});

		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache.expect_get_or_load().return_const(HANDLE);
				cache
			}
		}

		let mut app = setup::<_Factory>();
		let panel = app
			.world
			.spawn(QuickbarPanel {
				state: PanelState::Empty,
				key: arbitrary_key(),
			})
			.id();
		app.insert_resource(_IconPaths(vec![(panel, Some(Path::from("")))]));

		app.update();

		let panel = app.world.entity(panel);

		assert_eq!(
			(Some(HANDLE), Some(PanelState::Filled)),
			(
				panel.get::<UiImage>().map(|image| image.texture.clone()),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		)
	}

	#[test]
	fn load_image_with_correct_path() {
		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache
					.expect_get_or_load()
					.times(1)
					.with(eq(Path::from("image/path")))
					.return_const(Handle::default());
				cache
			}
		}

		let mut app = setup::<_Factory>();
		let panel = app
			.world
			.spawn(QuickbarPanel {
				state: PanelState::Empty,
				key: arbitrary_key(),
			})
			.id();
		app.insert_resource(_IconPaths(vec![(panel, Some(Path::from("image/path")))]));

		app.update();
	}

	#[test]
	fn set_panel_empty_when_icon_path_is_none() {
		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache
					.expect_get_or_load()
					.return_const(Handle::Weak(AssetId::Uuid {
						uuid: Uuid::new_v4(),
					}));
				cache
			}
		}

		let mut app = setup::<_Factory>();
		let panel = app
			.world
			.spawn(QuickbarPanel {
				state: PanelState::Filled,
				key: arbitrary_key(),
			})
			.id();
		app.insert_resource(_IconPaths(vec![(panel, None)]));

		app.update();

		let panel = app.world.entity(panel);

		assert_eq!(
			(Some(Handle::default()), Some(PanelState::Empty)),
			(
				panel.get::<UiImage>().map(|image| image.texture.clone()),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		);
	}
}
