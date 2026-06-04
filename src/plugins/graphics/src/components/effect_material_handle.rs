use crate::materials::effect_material::EffectMaterial;
use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Default)]
#[require(Visibility = Visibility::Hidden)]
pub struct EffectMaterialHandle {
	pub(crate) material: Handle<EffectMaterial>,
}

#[derive(Component, Default)]
#[relationship_target(relationship = EffectMeshOf)]
pub(crate) struct EffectMeshes(EntityHashSet);

#[derive(Component)]
#[relationship(relationship_target = EffectMeshes)]
pub(crate) struct EffectMeshOf(pub(crate) Entity);
