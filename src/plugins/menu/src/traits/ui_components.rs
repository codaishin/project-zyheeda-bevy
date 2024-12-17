use bevy::prelude::*;

pub trait GetZIndex {
	fn z_index(&self) -> Option<ZIndex> {
		None
	}
}

pub trait GetZIndexGlobal {
	fn z_index_global(&self) -> Option<GlobalZIndex> {
		None
	}
}

pub trait GetUIComponents {
	fn ui_components(&self) -> (Node, BackgroundColor);
}
