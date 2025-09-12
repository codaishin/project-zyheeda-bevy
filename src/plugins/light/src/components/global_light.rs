use bevy::prelude::*;
use common::traits::register_derived_component::{DerivableFrom, InsertDerivedComponent};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub(crate) struct GlobalLight(pub(crate) Srgba);

impl<'w, 's> DerivableFrom<'w, 's, GlobalLight> for DirectionalLight {
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;

	type TParam = ();

	fn derive_from(_: Entity, GlobalLight(color): &GlobalLight, _: &()) -> Option<Self> {
		Some(DirectionalLight {
			shadows_enabled: false,
			illuminance: 100.,
			color: Color::from(*color),
			..default()
		})
	}
}
