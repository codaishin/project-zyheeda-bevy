use super::get_combos::CombosDescriptor;
use crate::traits::SkillDescriptor;
use bevy::{
	asset::Handle,
	prelude::{In, ResMut, Resource},
	render::texture::Image,
};
use common::traits::{
	cache::{GetOrLoadAsset, GetOrLoadAssetFactory},
	load_asset::Path,
};

pub(crate) fn load_combo_icon_image<
	TKey: Clone,
	TAssets: Resource,
	TStorage: Resource,
	TFactory: GetOrLoadAssetFactory<TAssets, Image, TStorage>,
>(
	combos: In<CombosDescriptor<TKey, Path>>,
	assets: ResMut<TAssets>,
	storage: ResMut<TStorage>,
) -> CombosDescriptor<TKey, Handle<Image>> {
	let mut cache = TFactory::create_from(assets, storage);

	combos
		.0
		.iter()
		.map(|c| {
			c.iter()
				.map(|s| SkillDescriptor {
					name: s.name,
					key: s.key.clone(),
					icon: s.icon.clone().map(|icon| cache.get_or_load(icon)),
				})
				.collect()
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::SkillDescriptor;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		prelude::{Commands, IntoSystem, KeyCode, ResMut},
		utils::Uuid,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{cache::GetOrLoadAsset, load_asset::Path},
	};
	use mockall::{mock, predicate::eq};

	#[derive(Resource, Default)]
	struct _Storage;

	#[derive(Resource, Default)]
	struct _Assets;

	mock! {
		_Cache {}
		impl GetOrLoadAsset<Image> for _Cache {
			fn get_or_load(&mut self, key: Path) -> Handle<Image> {
				todo!()
			}
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(CombosDescriptor<KeyCode, Handle<Image>>);

	fn setup<TFactory: GetOrLoadAssetFactory<_Assets, Image, _Storage> + 'static>(
		combos: CombosDescriptor<KeyCode, Path>,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Assets>();
		app.init_resource::<_Storage>();
		app.add_systems(
			Update,
			(move || combos.clone())
				.pipe(load_combo_icon_image::<KeyCode, _Assets, _Storage, TFactory>)
				.pipe(
					|combos: In<CombosDescriptor<KeyCode, Handle<Image>>>,
					 mut commands: Commands| commands.insert_resource(_Result(combos.0)),
				),
		);

		app
	}

	#[test]
	fn load_icon() {
		struct _Factory;

		const HANDLE_A_1: Handle<Image> = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(0x5aa1803b_9027_4d84_99ab_6e2bc1420ba8),
		});
		const HANDLE_A_2: Handle<Image> = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(0x231707c1_bb0b_4e74_ab1e_c8de763a3190),
		});
		const HANDLE_B_1: Handle<Image> = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(0x5273f365_f464_434e_b4eb_5aca2b44a3ef),
		});
		const HANDLE_B_2: Handle<Image> = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(0x3ba8bc23_6c98_4730_bb60_9fdb71b853f1),
		});

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache
					.expect_get_or_load()
					.with(eq(Path::from("a/1")))
					.return_const(HANDLE_A_1);
				cache
					.expect_get_or_load()
					.with(eq(Path::from("a/2")))
					.return_const(HANDLE_A_2);
				cache
					.expect_get_or_load()
					.with(eq(Path::from("b/1")))
					.return_const(HANDLE_B_1);
				cache
					.expect_get_or_load()
					.with(eq(Path::from("b/2")))
					.return_const(HANDLE_B_2);

				cache
			}
		}

		let mut app = setup::<_Factory>(vec![
			vec![
				SkillDescriptor {
					name: "a1",
					key: KeyCode::KeyA,
					icon: Some(Path::from("a/1")),
				},
				SkillDescriptor {
					name: "a2",
					key: KeyCode::KeyB,
					icon: Some(Path::from("a/2")),
				},
			],
			vec![
				SkillDescriptor {
					name: "b1",
					key: KeyCode::KeyC,
					icon: Some(Path::from("b/1")),
				},
				SkillDescriptor {
					name: "b2",
					key: KeyCode::KeyD,
					icon: Some(Path::from("b/2")),
				},
			],
		]);

		app.update();

		let result = app.world.resource::<_Result>();

		assert_eq!(
			&_Result(vec![
				vec![
					SkillDescriptor {
						name: "a1",
						key: KeyCode::KeyA,
						icon: Some(HANDLE_A_1),
					},
					SkillDescriptor {
						name: "a2",
						key: KeyCode::KeyB,
						icon: Some(HANDLE_A_2),
					}
				],
				vec![
					SkillDescriptor {
						name: "b1",
						key: KeyCode::KeyC,
						icon: Some(HANDLE_B_1),
					},
					SkillDescriptor {
						name: "b2",
						key: KeyCode::KeyD,
						icon: Some(HANDLE_B_2),
					}
				]
			]),
			result
		)
	}
}
