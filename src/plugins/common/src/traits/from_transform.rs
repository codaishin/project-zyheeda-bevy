use bevy::transform::{components::Transform, TransformBundle};

pub trait FromTransform {
	fn from_transform(transform: Transform) -> Self;
}

impl FromTransform for TransformBundle {
	fn from_transform(transform: Transform) -> Self {
		TransformBundle::from_transform(transform)
	}
}
