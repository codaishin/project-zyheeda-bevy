use crate::components::{Floating, Light};
use bevy::{
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	pbr::{NotShadowCaster, PbrBundle, PointLight, PointLightBundle, StandardMaterial},
	render::{color::Color, view::VisibilityBundle},
	transform::components::Transform,
	utils::default,
};
use common::errors::Error;
use prefabs::traits::{sphere, AssetHandles, Instantiate};

impl Instantiate for Light<Floating> {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl AssetHandles,
	) -> Result<(), Error> {
		let radius = 0.1;
		let mesh = assets.handle::<Light<Floating>>(&|| sphere(radius));
		let material = assets.handle::<Light<Floating>>(&|| StandardMaterial {
			base_color: Color::WHITE,
			emissive: Color::rgb_linear(23000.0, 23000.0, 23000.0),
			..default()
		});
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
