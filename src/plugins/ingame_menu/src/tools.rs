pub(crate) mod menu_state;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelState {
	Empty,
	Filled,
}

#[cfg(test)]
#[derive(Debug, PartialEq)]
pub(crate) struct HasNodeBundleComponents {
	pub node: bool,
	pub style: bool,
	pub background_color: bool,
	pub border_color: bool,
	pub focus_policy: bool,
	pub transform: bool,
	pub global_transform: bool,
	pub visibility: bool,
	pub inherited_visibility: bool,
	pub view_visibility: bool,
	pub z_index: bool,
}

#[cfg(test)]
macro_rules! assert_node_bundle {
	($entity:expr) => {
		assert_eq!(
			crate::tools::HasNodeBundleComponents {
				node: true,
				style: true,
				background_color: true,
				border_color: true,
				focus_policy: true,
				transform: true,
				global_transform: true,
				visibility: true,
				inherited_visibility: true,
				view_visibility: true,
				z_index: true,
			},
			crate::tools::HasNodeBundleComponents {
				node: $entity.contains::<bevy::prelude::Node>(),
				style: $entity.contains::<bevy::prelude::Style>(),
				background_color: $entity.contains::<bevy::prelude::BackgroundColor>(),
				border_color: $entity.contains::<bevy::prelude::BorderColor>(),
				focus_policy: $entity.contains::<bevy::ui::FocusPolicy>(),
				transform: $entity.contains::<bevy::prelude::Transform>(),
				global_transform: $entity.contains::<bevy::prelude::GlobalTransform>(),
				visibility: $entity.contains::<bevy::prelude::Visibility>(),
				inherited_visibility: $entity.contains::<bevy::prelude::InheritedVisibility>(),
				view_visibility: $entity.contains::<bevy::prelude::ViewVisibility>(),
				z_index: $entity.contains::<bevy::prelude::ZIndex>(),
			}
		)
	};
}

#[cfg(test)]
pub(crate) use assert_node_bundle;
