use crate::{
	components::material_override::MaterialOverride,
	materials::essence_material::EssenceMaterial,
};
use bevy::prelude::*;
use common::traits::{
	add_asset::AddAsset,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};

impl MaterialOverride {
	pub(crate) fn update_essence_shader(
		trigger: Trigger<OnInsert, Self>,
		commands: Commands,
		assets: ResMut<Assets<EssenceMaterial>>,
		overrides: Query<&Self>,
		with_active_standard_material: Query<&MeshMaterial3d<StandardMaterial>>,
		with_inactive_standard_material: Query<&inactive::Material>,
	) {
		Self::update_essence_shader_internal(
			trigger,
			commands,
			assets,
			overrides,
			with_active_standard_material,
			with_inactive_standard_material,
		);
	}

	fn update_essence_shader_internal<TAssets>(
		trigger: Trigger<OnInsert, Self>,
		commands: Commands,
		assets: ResMut<TAssets>,
		overrides: Query<&Self>,
		with_active_standard_material: Query<&MeshMaterial3d<StandardMaterial>>,
		with_inactive_standard_material: Query<&inactive::Material>,
	) where
		TAssets: AddAsset<EssenceMaterial> + Resource,
	{
		let entity = trigger.target();

		let Ok(material_override) = overrides.get(entity) else {
			return;
		};

		match material_override {
			MaterialOverride::None => {
				set_standard_material(commands, with_inactive_standard_material, entity);
			}
			MaterialOverride::Material(essence_material) => {
				set_essence_material(
					commands,
					assets,
					with_active_standard_material,
					entity,
					essence_material,
				);
			}
		}
	}
}

fn set_standard_material(
	mut commands: Commands,
	with_inactive_standard_material: Query<&inactive::Material>,
	entity: Entity,
) {
	commands.try_remove_from::<MeshMaterial3d<EssenceMaterial>>(entity);

	let Ok(inactive::Material(material)) = with_inactive_standard_material.get(entity) else {
		return;
	};

	commands.try_insert_on(entity, MeshMaterial3d(material.clone()));
}

fn set_essence_material<TAssets>(
	mut commands: Commands,
	mut assets: ResMut<TAssets>,
	with_active_standard_material: Query<&MeshMaterial3d<StandardMaterial>>,
	entity: Entity,
	essence_material: &EssenceMaterial,
) where
	TAssets: Resource + AddAsset<EssenceMaterial>,
{
	commands.try_insert_on(
		entity,
		MeshMaterial3d(assets.add_asset(essence_material.clone())),
	);
	commands.try_remove_from::<MeshMaterial3d<StandardMaterial>>(entity);

	let Ok(MeshMaterial3d(material)) = with_active_standard_material.get(entity) else {
		return;
	};

	commands.try_insert_on(entity, inactive::Material(material.clone()));
}

mod inactive {
	use super::*;

	#[derive(Component)]
	pub struct Material(pub(super) Handle<StandardMaterial>);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::materials::essence_material::EssenceMaterial;
	use bevy::color::palettes::css::RED;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, new_handle};

	#[derive(Resource, NestedMocks)]
	struct _Assets {
		mock: Mock_Assets,
	}

	#[automock]
	impl AddAsset<EssenceMaterial> for _Assets {
		fn add_asset(&mut self, asset: EssenceMaterial) -> Handle<EssenceMaterial> {
			self.mock.add_asset(asset)
		}
	}

	fn setup(assets: _Assets) -> App {
		let mut app = App::new();

		app.add_observer(MaterialOverride::update_essence_shader_internal::<_Assets>);
		app.insert_resource(assets);

		app
	}

	#[test]
	fn insert_essence_material() {
		let material = EssenceMaterial {
			texture_color: RED.into(),
			..default()
		};
		let handle = new_handle();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset()
				.times(1)
				.with(eq(material.clone()))
				.return_const(handle.clone());
		}));

		let entity = app.world_mut().spawn((
			MeshMaterial3d(new_handle::<StandardMaterial>()),
			MaterialOverride::Material(material),
		));

		assert_eq!(
			Some(&MeshMaterial3d(handle)),
			entity.get::<MeshMaterial3d<EssenceMaterial>>(),
		);
	}

	#[test]
	fn remove_standard_material() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().return_const(new_handle());
		}));

		let entity = app.world_mut().spawn((
			MeshMaterial3d(new_handle::<StandardMaterial>()),
			MaterialOverride::Material(EssenceMaterial::default()),
		));

		assert_eq!(
			None,
			entity
				.get::<MeshMaterial3d<StandardMaterial>>()
				.map(|m| &m.0)
		);
	}

	#[test]
	fn remove_essence_material_when_set_to_standard_material() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().never().return_const(new_handle());
		}));

		let entity = app.world_mut().spawn((
			MeshMaterial3d(new_handle::<EssenceMaterial>()),
			MaterialOverride::None,
		));

		assert_eq!(None, entity.get::<MeshMaterial3d<EssenceMaterial>>());
	}

	#[test]
	fn re_add_standard_material_when_set_to_standard_material() {
		let original_material = new_handle::<StandardMaterial>();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().return_const(new_handle());
		}));
		let mut entity = app.world_mut().spawn((
			MeshMaterial3d(original_material.clone()),
			MaterialOverride::Material(EssenceMaterial::default()),
		));

		entity.insert(MaterialOverride::None);

		assert_eq!(
			Some(&original_material),
			entity
				.get::<MeshMaterial3d<StandardMaterial>>()
				.map(|m| &m.0),
		);
	}
}
