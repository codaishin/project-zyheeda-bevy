use crate::{
	materials::lit_material::{LitMaterial, StandardLitMaterial},
	resources::standard_materials::StandardMaterials,
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl StandardMaterials {
	pub(crate) fn replace_with_lit_material(
		mut materials: ResMut<Self>,
		standard_materials: Res<Assets<StandardMaterial>>,
		mut lit_materials: ResMut<Assets<StandardLitMaterial>>,
		mut commands: ZyheedaCommands,
	) {
		materials.entities.retain(|id, entities| {
			let Some(base) = standard_materials.get(*id).cloned() else {
				return true;
			};

			let lit_material = lit_materials.add(StandardLitMaterial {
				base,
				extension: LitMaterial::default(),
			});

			for entity in entities.iter() {
				commands.try_apply_on(entity, |mut e| {
					e.try_remove::<MeshMaterial3d<StandardMaterial>>();
					e.try_insert(MeshMaterial3d(lit_material.clone()));
				});
			}

			false
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::materials::lit_material::StandardLitMaterial;
	use std::collections::{HashMap, HashSet};
	use testing::{SingleThreadedApp, new_handle};

	fn setup<const N: usize>(materials: [(&Handle<StandardMaterial>, StandardMaterial); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, asset) in materials {
			_ = assets.insert(id, asset);
		}

		app.insert_resource(assets);
		app.init_resource::<Assets<StandardLitMaterial>>();
		app.add_systems(Update, StandardMaterials::replace_with_lit_material);

		app
	}

	#[test]
	fn replace() {
		let handle = new_handle();
		let mut app = setup([(
			&handle,
			StandardMaterial {
				base_color: Color::LinearRgba(LinearRgba::new(4., 3., 2., 1.)),
				..default()
			},
		)]);
		let entity = app.world_mut().spawn(MeshMaterial3d(handle.clone())).id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), HashSet::from([entity]))]),
		});

		app.update();

		assert_eq!(
			(
				&StandardMaterials::default(),
				Some((
					Color::LinearRgba(LinearRgba::new(4., 3., 2., 1.)),
					LitMaterial::default()
				)),
				None
			),
			(
				app.world().resource::<StandardMaterials>(),
				app.world()
					.entity(entity)
					.get::<MeshMaterial3d<StandardLitMaterial>>()
					.and_then(|MeshMaterial3d(handle)| app
						.world()
						.resource::<Assets<StandardLitMaterial>>()
						.get(handle))
					.map(|m| (m.base.base_color, m.extension)),
				app.world()
					.entity(entity)
					.get::<MeshMaterial3d<StandardMaterial>>()
			)
		);
	}

	#[test]
	fn replace_shared() {
		let handle = new_handle();
		let mut app = setup([(
			&handle,
			StandardMaterial {
				base_color: Color::LinearRgba(LinearRgba::new(4., 3., 2., 1.)),
				..default()
			},
		)]);
		let a = app.world_mut().spawn(MeshMaterial3d(handle.clone())).id();
		let b = app.world_mut().spawn(MeshMaterial3d(handle.clone())).id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), HashSet::from([a, b]))]),
		});

		app.update();

		assert_eq!(
			app.world()
				.entity(a)
				.get::<MeshMaterial3d<StandardLitMaterial>>(),
			app.world()
				.entity(b)
				.get::<MeshMaterial3d<StandardLitMaterial>>(),
		);
	}

	#[test]
	fn replace_delayed() {
		let handle = new_handle();
		let mut app = setup([]);
		let entity = app.world_mut().spawn(MeshMaterial3d(handle.clone())).id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), HashSet::from([entity]))]),
		});

		app.update();
		_ = app
			.world_mut()
			.resource_mut::<Assets<StandardMaterial>>()
			.insert(
				&handle,
				StandardMaterial {
					base_color: Color::LinearRgba(LinearRgba::new(4., 3., 2., 1.)),
					..default()
				},
			);
		app.update();

		assert_eq!(
			(
				&StandardMaterials::default(),
				Some(Color::LinearRgba(LinearRgba::new(4., 3., 2., 1.))),
				None
			),
			(
				app.world().resource::<StandardMaterials>(),
				app.world()
					.entity(entity)
					.get::<MeshMaterial3d<StandardLitMaterial>>()
					.and_then(|MeshMaterial3d(handle)| app
						.world()
						.resource::<Assets<StandardLitMaterial>>()
						.get(handle))
					.map(|m| m.base.base_color),
				app.world()
					.entity(entity)
					.get::<MeshMaterial3d<StandardMaterial>>()
			)
		);
	}

	#[test]
	fn leave_standard_assets_untouched() {
		let handle = new_handle();
		let mut app = setup([(
			&handle,
			StandardMaterial {
				base_color: Color::LinearRgba(LinearRgba::new(4., 3., 2., 1.)),
				..default()
			},
		)]);
		let entity = app.world_mut().spawn(MeshMaterial3d(handle.clone())).id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), HashSet::from([entity]))]),
		});

		app.update();

		assert_eq!(
			vec![(
				handle.id(),
				Color::LinearRgba(LinearRgba::new(4., 3., 2., 1.))
			)],
			app.world()
				.resource::<Assets<StandardMaterial>>()
				.iter()
				.map(|(id, a)| (id, a.base_color))
				.collect::<Vec<_>>(),
		);
	}
}
