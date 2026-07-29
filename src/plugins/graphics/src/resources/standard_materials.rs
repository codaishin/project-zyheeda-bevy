use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::materials::lit_material::DeadSpace;

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct StandardMaterials {
	pub(crate) entities: HashMap<AssetId<StandardMaterial>, (HashSet<Entity>, DeadSpace)>,
}
