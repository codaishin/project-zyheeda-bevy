use bevy::{
	color::Color,
	math::Vec3,
	pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial},
	prelude::{default, ChildBuilder, Transform},
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, Sensor};
use common::{
	bundles::ColliderTransformBundle,
	components::ColliderRoot,
	tools::{Intensity, Units},
	traits::{
		cache::GetOrCreateTypeAsset,
		prefab::{sphere, GetOrCreateAssets},
	},
};

pub(super) trait ProjectileTypeParameters {
	fn radius() -> Units;
	fn base_color() -> Color;
	fn emissive() -> (Color, Intensity);
}

pub(super) trait ProjectileSubtype {
	fn spawn_contact(self, parent: &mut ChildBuilder, assets: &mut impl GetOrCreateAssets);
	fn spawn_projection(self, parent: &mut ChildBuilder);
}

impl<T: ProjectileTypeParameters + 'static> ProjectileSubtype for T {
	fn spawn_contact(self, parent: &mut ChildBuilder, assets: &mut impl GetOrCreateAssets) {
		let transform = Transform::from_translation(Vec3::ZERO);
		let radius = *T::radius();
		let (emissive_color, emissive_intensity) = T::emissive();

		let mesh = assets.get_or_create_for::<T>(|| sphere(radius));
		let material = assets.get_or_create_for::<T>(|| StandardMaterial {
			emissive: emissive_color.to_linear() * *emissive_intensity,
			base_color: T::base_color(),
			..default()
		});

		parent.spawn(PbrBundle {
			mesh,
			material,
			transform,
			..default()
		});
		parent.spawn((
			ColliderTransformBundle {
				transform,
				collider: Collider::ball(radius),
				active_events: ActiveEvents::COLLISION_EVENTS,
				..default()
			},
			Sensor,
			ColliderRoot(parent.parent_entity()),
		));
		parent.spawn(PointLightBundle {
			point_light: PointLight {
				color: emissive_color,
				intensity: 8000.,
				shadows_enabled: true,
				..default()
			},
			..default()
		});
	}

	fn spawn_projection(self, parent: &mut ChildBuilder) {
		parent.spawn((
			ColliderTransformBundle {
				collider: Collider::ball(*T::radius()),
				active_events: ActiveEvents::COLLISION_EVENTS,
				..default()
			},
			Sensor,
			ColliderRoot(parent.parent_entity()),
		));
	}
}
