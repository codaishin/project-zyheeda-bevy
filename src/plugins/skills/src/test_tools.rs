#[cfg(test)]
#[derive(Debug, PartialEq)]
pub(crate) struct HasSpacialComponent {
	pub visibility: bool,
	pub inherited_visibility: bool,
	pub view_visibility: bool,
	pub transform: bool,
	pub global_transform: bool,
}

#[cfg(test)]
macro_rules! assert_spacial_bundle {
	($entity:expr) => {
		assert_eq!(
			crate::test_tools::HasSpacialComponent {
				visibility: true,
				inherited_visibility: true,
				view_visibility: true,
				transform: true,
				global_transform: true,
			},
			crate::test_tools::HasSpacialComponent {
				visibility: $entity.contains::<bevy::render::view::Visibility>(),
				inherited_visibility: $entity.contains::<bevy::render::view::InheritedVisibility>(),
				view_visibility: $entity.contains::<bevy::render::view::ViewVisibility>(),
				transform: $entity.contains::<bevy::transform::components::Transform>(),
				global_transform: $entity
					.contains::<bevy::transform::components::GlobalTransform>(),
			}
		)
	};
}

#[cfg(test)]
pub(crate) use assert_spacial_bundle;
