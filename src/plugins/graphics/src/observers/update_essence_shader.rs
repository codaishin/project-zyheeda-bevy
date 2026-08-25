use crate::{
	components::material_override::MaterialOverride,
	materials::{essence_material::EssenceMaterial, lit_material::StandardLitMaterial},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::prelude::*;

impl MaterialOverride {
	pub(crate) fn update_essence_shader(
		on_insert: On<Insert, Self>,
		commands: ZyheedaCommands,
		assets: ResMut<Assets<EssenceMaterial>>,
		overrides: Query<&Self>,
		with_active_standard_material: Query<&MeshMaterial3d<StandardMaterial>>,
		with_active_lit_material: Query<&MeshMaterial3d<StandardLitMaterial>>,
		with_inactive_material: Query<&inactive::Material>,
	) {
		Self::update_essence_shader_internal(
			on_insert,
			commands,
			assets,
			overrides,
			with_active_standard_material,
			with_active_lit_material,
			with_inactive_material,
		);
	}

	fn update_essence_shader_internal<TAssets>(
		on_insert: On<Insert, Self>,
		commands: ZyheedaCommands,
		assets: ResMut<TAssets>,
		overrides: Query<&Self>,
		with_active_standard_material: Query<&MeshMaterial3d<StandardMaterial>>,
		with_active_lit_material: Query<&MeshMaterial3d<StandardLitMaterial>>,
		with_inactive_material: Query<&inactive::Material>,
	) where
		TAssets: AddAsset<EssenceMaterial> + Resource<Mutability = Mutable>,
	{
		let entity = on_insert.entity;

		let Ok(material_override) = overrides.get(entity) else {
			return;
		};

		match material_override {
			MaterialOverride::None => {
				set_standard_material(commands, with_inactive_material, entity);
			}
			MaterialOverride::Material(essence_material) => {
				set_essence_material(
					commands,
					assets,
					with_active_standard_material,
					with_active_lit_material,
					entity,
					essence_material,
				);
			}
		}
	}
}

fn set_standard_material(
	mut commands: ZyheedaCommands,
	with_inactive_standard_material: Query<&inactive::Material>,
	entity: Entity,
) {
	commands.try_apply_on(&entity, |mut e| {
		e.try_remove::<MeshMaterial3d<EssenceMaterial>>();

		let Ok(material) = with_inactive_standard_material.get(e.id()) else {
			return;
		};

		match material {
			inactive::Material::Unlit(material) => {
				e.try_insert(MeshMaterial3d(material.clone()));
			}
			inactive::Material::Lit(material) => {
				e.try_insert(MeshMaterial3d(material.clone()));
			}
		}
	});
}

fn set_essence_material<TAssets>(
	mut commands: ZyheedaCommands,
	mut assets: ResMut<TAssets>,
	with_active_standard_material: Query<&MeshMaterial3d<StandardMaterial>>,
	with_active_lit_material: Query<&MeshMaterial3d<StandardLitMaterial>>,
	entity: Entity,
	essence_material: &EssenceMaterial,
) where
	TAssets: Resource<Mutability = Mutable> + AddAsset<EssenceMaterial>,
{
	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(MeshMaterial3d(assets.add_asset(essence_material.clone())));
		e.try_remove::<MeshMaterial3d<StandardMaterial>>();
		e.try_remove::<MeshMaterial3d<StandardLitMaterial>>();

		if let Ok(MeshMaterial3d(material)) = with_active_standard_material.get(e.id()) {
			e.try_insert(inactive::Material::Unlit(material.clone()));
		};

		if let Ok(MeshMaterial3d(material)) = with_active_lit_material.get(e.id()) {
			e.try_insert(inactive::Material::Lit(material.clone()));
		};
	});
}

mod inactive {
	use super::*;

	#[derive(Component)]
	pub enum Material {
		Unlit(Handle<StandardMaterial>),
		Lit(Handle<StandardLitMaterial>),
	}
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
	fn remove_standard_lit_material() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().return_const(new_handle());
		}));

		let entity = app.world_mut().spawn((
			MeshMaterial3d(new_handle::<StandardLitMaterial>()),
			MaterialOverride::Material(EssenceMaterial::default()),
		));

		assert_eq!(
			None,
			entity
				.get::<MeshMaterial3d<StandardLitMaterial>>()
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

	#[test]
	fn re_add_standard_lit_material_when_set_to_standard_material() {
		let original_material = new_handle::<StandardLitMaterial>();
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
				.get::<MeshMaterial3d<StandardLitMaterial>>()
				.map(|m| &m.0),
		);
	}
}
