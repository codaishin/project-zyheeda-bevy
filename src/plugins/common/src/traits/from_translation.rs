use bevy::{
	math::Vec3,
	transform::{components::Transform, TransformBundle},
};

pub trait FromTranslation {
	fn from_translation(translation: Vec3) -> Self;
}

impl FromTranslation for TransformBundle {
	fn from_translation(translation: Vec3) -> Self {
		TransformBundle::from_transform(Transform::from_translation(translation))
	}
}
