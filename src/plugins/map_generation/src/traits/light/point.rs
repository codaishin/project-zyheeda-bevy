use crate::components::{Light, Point};
use bevy::{
	asset::Handle,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	pbr::{NotShadowCaster, PbrBundle, PointLight, PointLightBundle, StandardMaterial},
	render::{color::Color, mesh::Mesh, view::VisibilityBundle},
	transform::components::Transform,
	utils::default,
};
use common::errors::Error;
use prefabs::traits::{sphere, AssetKey, Instantiate};

impl Instantiate for Light<Point> {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error> {
		let radius = 0.1;
		let mesh = get_mesh_handle(AssetKey::Light, sphere(radius));
		let material = get_material_handle(
			AssetKey::Light,
			StandardMaterial {
				base_color: Color::WHITE,
				emissive: Color::rgb_linear(23000.0, 23000.0, 23000.0),
				..default()
			},
		);
		let transform = Transform::from_xyz(0., 1.8, 0.);

		on.try_insert(VisibilityBundle::default())
			.with_children(|parent| {
				parent
					.spawn((
						PbrBundle {
							mesh,
							material,
							transform,
							..default()
						},
						NotShadowCaster,
					))
					.with_children(|parent| {
						parent.spawn((PointLightBundle {
							point_light: PointLight {
								shadows_enabled: true,
								intensity: 10_000.0,
								radius,
								..default()
							},
							..default()
						},));
					});
			});

		Ok(())
	}
}
