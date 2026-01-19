use crate::components::collider::Colliders;
use bevy::prelude::*;
use common::traits::handles_physics::physical_bodies::Body;

#[derive(Component, Debug, PartialEq)]
#[require(Colliders)]
#[component(immutable)]
pub struct PhysicalBody(pub(crate) Body);

impl From<Body> for PhysicalBody {
	fn from(body: Body) -> Self {
		Self(body)
	}
}
