use bevy::prelude::*;
use common::{
	components::AssetModel,
	traits::{add_asset::AddAsset, try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};
use shaders::materials::essence_material::EssenceMaterial;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Renderer {
	pub model: AssetModel,
	pub essence: EssenceRender,
}

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub enum EssenceRender {
	#[default]
	StandardMaterial,
	Material(EssenceMaterial),
}

impl EssenceRender {
	pub(crate) fn apply_material_exclusivity(
		commands: Commands,
		assets: ResMut<Assets<EssenceMaterial>>,
		essence_renders: Query<MaterialComponents>,
	) {
		apply_material_exclusivity(commands, assets, essence_renders);
	}
}

type MaterialComponents<'a> = (
	Entity,
	&'a EssenceRender,
	Option<&'a Handle<StandardMaterial>>,
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
			EssenceRender::StandardMaterial => {
				commands.try_remove_from::<Handle<EssenceMaterial>>(entity);

				activate_standard_material(&mut commands, entity, inactive);
			}
			EssenceRender::Material(essence_material) => {
				commands.try_insert_on(entity, assets.add_asset(essence_material.clone()));

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

	commands.try_insert_on(entity, material.clone());
}

fn deactivate_standard_material(
	commands: &mut Commands,
	entity: Entity,
	material: Option<&Handle<StandardMaterial>>,
) {
	let Some(material) = material else {
		return;
	};

	commands.try_remove_from::<Handle<StandardMaterial>>(entity);
	commands.try_insert_on(entity, Inactive(material.clone()));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{color::palettes::css::RED, ecs::system::RunSystemOnce};
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
		let entity = app
			.world_mut()
			.spawn((
				new_handle::<StandardMaterial>(),
				EssenceRender::Material(material),
			))
			.id();

		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>);

		assert_eq!(
			Some(&handle),
			app.world().entity(entity).get::<Handle<EssenceMaterial>>()
		)
	}

	#[test]
	fn remove_standard_material() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().return_const(new_handle());
		}));
		let entity = app
			.world_mut()
			.spawn((
				new_handle::<StandardMaterial>(),
				EssenceRender::Material(EssenceMaterial::default()),
			))
			.id();

		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>);

		assert_eq!(
			None,
			app.world().entity(entity).get::<Handle<StandardMaterial>>()
		)
	}

	#[test]
	fn remove_essence_material_when_set_to_standard_material() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().never().return_const(new_handle());
		}));
		let entity = app
			.world_mut()
			.spawn((
				new_handle::<EssenceMaterial>(),
				EssenceRender::StandardMaterial,
			))
			.id();

		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>);

		assert_eq!(
			None,
			app.world().entity(entity).get::<Handle<EssenceMaterial>>()
		)
	}

	#[test]
	fn re_add_standard_material_when_set_to_standard_material() {
		let original_material = new_handle::<StandardMaterial>();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_add_asset().return_const(new_handle());
		}));
		let entity = app
			.world_mut()
			.spawn((
				original_material.clone(),
				EssenceRender::Material(EssenceMaterial::default()),
			))
			.id();

		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>);
		let mut entity_ref = app.world_mut().entity_mut(entity);
		let mut essence_render = entity_ref.get_mut::<EssenceRender>().unwrap();
		*essence_render = EssenceRender::StandardMaterial;
		app.world_mut()
			.run_system_once(apply_material_exclusivity::<_Assets>);

		assert_eq!(
			Some(&original_material),
			app.world().entity(entity).get::<Handle<StandardMaterial>>()
		)
	}
}
