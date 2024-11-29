use crate::components::{Floating, Light};
use bevy::{
	color::Color,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	pbr::{NotShadowCaster, PbrBundle, PointLight, PointLightBundle, StandardMaterial},
	render::view::VisibilityBundle,
	transform::components::Transform,
	utils::default,
};
use common::{
	errors::Error,
	traits::{
		cache::GetOrCreateTypeAsset,
		prefab::{sphere, GetOrCreateAssets, Prefab},
	},
};

impl Prefab<()> for Light<Floating> {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let radius = 0.1;
		let mesh = assets.get_or_create_for::<Light<Floating>>(|| sphere(radius));
		let material = assets.get_or_create_for::<Light<Floating>>(|| StandardMaterial {
			base_color: Color::WHITE,
			emissive: Color::linear_rgb(230.0, 230.0, 230.0).into(),
			..default()
		});
		let transform = Transform::from_xyz(0., 1.8, 0.);

		entity
			.try_insert(VisibilityBundle::default())
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
