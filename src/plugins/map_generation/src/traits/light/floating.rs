use bevy::{pbr::NotShadowCaster, prelude::*};
use common::{
	components::{insert_asset::InsertAsset, spawn_children::SpawnChildren},
	traits::prefab::sphere,
};

#[derive(Component, Debug, PartialEq)]
#[require(SpawnChildren(Self::model))]
pub(crate) struct FloatingLight;

impl FloatingLight {
	fn model() -> SpawnChildren {
		SpawnChildren(|parent| {
			parent.spawn(FloatingLightModel);
		})
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(
	Visibility,
	Transform(Self::transform),
	InsertAsset<Mesh>(Self::mesh),
	InsertAsset<StandardMaterial>(Self::material),
	NotShadowCaster,
	SpawnChildren(Self::point_light),
)]
pub(crate) struct FloatingLightModel;

impl FloatingLightModel {
	const RADIUS: f32 = 0.1;

	fn transform() -> Transform {
		Transform::from_xyz(0., 1.8, 0.)
	}

	fn mesh() -> InsertAsset<Mesh> {
		InsertAsset::shared::<Self>(|| sphere(Self::RADIUS))
	}

	fn material() -> InsertAsset<StandardMaterial> {
		InsertAsset::shared::<Self>(|| StandardMaterial {
			base_color: Color::WHITE,
			emissive: Color::linear_rgb(230.0, 230.0, 230.0).into(),
			..default()
		})
	}

	fn point_light() -> SpawnChildren {
		SpawnChildren(|parent| {
			parent.spawn(PointLight {
				shadows_enabled: true,
				intensity: 10_000.0,
				radius: Self::RADIUS,
				..default()
			});
		})
	}
}
