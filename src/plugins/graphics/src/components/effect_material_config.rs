use crate::materials::effect_material::EffectMaterial;
use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Default)]
#[require(Visibility = Visibility::Hidden)]
pub struct EffectShader {
	pub(crate) material: Handle<EffectMaterial>,
}

#[derive(Component, Default)]
#[relationship_target(relationship = EffectShaderMeshOf)]
pub(crate) struct EffectShaderMeshes(EntityHashSet);

#[derive(Component)]
#[relationship(relationship_target = EffectShaderMeshes)]
pub(crate) struct EffectShaderMeshOf(pub(crate) Entity);
