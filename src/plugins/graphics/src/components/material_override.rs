use crate::materials::essence_material::EssenceMaterial;
use bevy::{
	color::palettes::{
		css::LIGHT_CYAN,
		tailwind::{CYAN_100, CYAN_200},
	},
	prelude::*,
};
use common::{
	components::essence::Essence,
	traits::{add_asset::AddAsset, try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub enum MaterialOverride {
	#[default]
	None,
	Material(EssenceMaterial),
}

impl MaterialOverride {
	pub(crate) fn apply_material_exclusivity(
		commands: Commands,
		assets: ResMut<Assets<EssenceMaterial>>,
		essence_renders: Query<MaterialComponents>,
	) {
		apply_material_exclusivity(commands, assets, essence_renders);
	}
}

impl From<&Essence> for MaterialOverride {
	fn from(essence: &Essence) -> Self {
		match essence {
			Essence::None => MaterialOverride::None,
			Essence::Force => MaterialOverride::Material(EssenceMaterial {
				texture_color: CYAN_100.into(),
				fill_color: CYAN_200.into(),
				fresnel_color: (LIGHT_CYAN * 1.5).into(),
				..default()
			}),
		}
	}
}

type MaterialComponents<'a> = (
	Entity,
	&'a MaterialOverride,
	Option<&'a MeshMaterial3d<StandardMaterial>>,
	Option<&'a Inactive>,
);

#[derive(Component)]
pub struct Inactive(Handle<StandardMaterial>);

fn apply_material_exclusivity<TAssets>(
	mut commands: Commands,
	mut assets: ResMut<TAssets>,
	essence_renders: Query<MaterialComponents>,
) where
	TAssets: AddAsset<EssenceMaterial> + Resource,
{
	for (entity, essence_render, active, inactive) in &essence_renders {
		match essence_render {
			MaterialOverride::None => {
				commands.try_remove_from::<MeshMaterial3d<EssenceMaterial>>(entity);

				activate_standard_material(&mut commands, entity, inactive);
			}
			MaterialOverride::Material(essence_material) => {
				commands.try_insert_on(
					entity,
					MeshMaterial3d(assets.add_asset(essence_material.clone())),
				);

				deactivate_standard_material(&mut commands, entity, active);
			}
		}
	}
}

fn activate_standard_material(
	commands: &mut Commands,
	entity: Entity,
	inactive: Option<&Inactive>,
) {
	let Some(Inactive(material)) = inactive else {
		return;
	};

	commands.try_insert_on(entity, MeshMaterial3d(material.clone()));
}

fn deactivate_standard_material(
	commands: &mut Commands,
	entity: Entity,
	material: Option<&MeshMaterial3d<StandardMaterial>>,
) {
	let Some(MeshMaterial3d(material)) = material else {
		return;
	};

	commands.try_remove_from::<MeshMaterial3d<StandardMaterial>>(entity);
	commands.try_insert_on(entity, Inactive(material.clone()));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		color::palettes::css::RED,
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{test_tools::utils::new_handle, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

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
		app.insert_resource(assets);
		app
	}

	#[test]
	fn insert_essence_material() -> Result<(), RunSystemError> {
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
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d(new_handle::<StandardMaterial>()),
				MaterialOverride::Material(material),
			))
			.id();

		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>)?;

		assert_eq!(
			Some(&MeshMaterial3d(handle)),
			app.world()
				.entity(entity)
				.get::<MeshMaterial3d<EssenceMaterial>>()
		);
		Ok(())
	}

	#[test]
	fn remove_standard_material() -> Result<(), RunSystemError> {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().return_const(new_handle());
		}));
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d(new_handle::<StandardMaterial>()),
				MaterialOverride::Material(EssenceMaterial::default()),
			))
			.id();

		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>)?;

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<MeshMaterial3d<StandardMaterial>>()
				.map(|m| &m.0)
		);
		Ok(())
	}

	#[test]
	fn remove_essence_material_when_set_to_standard_material() -> Result<(), RunSystemError> {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().never().return_const(new_handle());
		}));
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d(new_handle::<EssenceMaterial>()),
				MaterialOverride::None,
			))
			.id();

		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>)?;

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<MeshMaterial3d<EssenceMaterial>>()
		);
		Ok(())
	}

	#[test]
	fn re_add_standard_material_when_set_to_standard_material() -> Result<(), RunSystemError> {
		let original_material = new_handle::<StandardMaterial>();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().return_const(new_handle());
		}));
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d(original_material.clone()),
				MaterialOverride::Material(EssenceMaterial::default()),
			))
			.id();

		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>)?;
		let mut entity_ref = app.world_mut().entity_mut(entity);
		let mut essence_render = entity_ref.get_mut::<MaterialOverride>().unwrap();
		*essence_render = MaterialOverride::None;
		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>)?;

		assert_eq!(
			Some(&original_material),
			app.world()
				.entity(entity)
				.get::<MeshMaterial3d<StandardMaterial>>()
				.map(|m| &m.0)
		);
		Ok(())
	}
}
