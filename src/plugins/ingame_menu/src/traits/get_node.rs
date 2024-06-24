use bevy::ui::node_bundles::NodeBundle;

pub trait GetNode {
	fn node(&self) -> NodeBundle;
}
