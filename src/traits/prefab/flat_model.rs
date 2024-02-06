use crate::{bundles::ColliderBundle, resources::Prefab};
use bevy::pbr::PbrBundle;

pub type FlatPrefab<TFor, TParent, TExtra> =
	Prefab<TFor, TParent, (PbrBundle, ColliderBundle<TExtra>)>;
