use crate::{bundles::ColliderBundle, resources::Prefab};
use bevy::pbr::PbrBundle;

pub type FlatPrefab<TParent, TExtra> = Prefab<TParent, (PbrBundle, ColliderBundle<TExtra>)>;
