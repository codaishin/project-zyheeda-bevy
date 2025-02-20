use crate::components::{Floating, Light};
use bevy::{pbr::NotShadowCaster, prelude::*};
use common::{
	components::asset_component::AssetComponent,
	errors::Error,
	traits::prefab::{sphere, Prefab},
};

impl Prefab<()> for Light<Floating> {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
	) -> Result<(), Error> {
		let radius = 0.1;
		entity.with_children(|parent| {
			parent.spawn(FloatingLightModel).with_children(|parent| {
				parent.spawn(PointLight {
					shadows_enabled: true,
					intensity: 10_000.0,
					radius,
					..default()
				});
			});
		});

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(
	Visibility,
	Transform(Self::transform),
	AssetComponent<Mesh>(Self::mesh),
	AssetComponent<StandardMaterial>(Self::material),
	NotShadowCaster,
)]
pub(crate) struct FloatingLightModel;

impl FloatingLightModel {
	fn transform() -> Transform {
		Transform::from_xyz(0., 1.8, 0.)
	}

	fn mesh() -> AssetComponent<Mesh> {
		AssetComponent::shared::<Self>(|| sphere(0.1))
	}

	fn material() -> AssetComponent<StandardMaterial> {
		AssetComponent::shared::<Self>(|| StandardMaterial {
			base_color: Color::WHITE,
			emissive: Color::linear_rgb(230.0, 230.0, 230.0).into(),
			..default()
		})
	}
}
