use crate::item::Item;
use bevy::prelude::*;

pub(crate) trait VisualizeItem {
	type TComponent: Component;

	fn visualize(item: Option<&Item>) -> Self::TComponent;
}
