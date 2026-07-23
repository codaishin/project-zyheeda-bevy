use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct StandardMaterials {
	pub(crate) entities: HashMap<AssetId<StandardMaterial>, HashSet<Entity>>,
}
