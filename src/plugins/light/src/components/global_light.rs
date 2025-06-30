use bevy::prelude::*;
use common::traits::register_derived_component::{DerivableComponentFrom, InsertDerivedComponent};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub(crate) struct GlobalLight(pub(crate) Srgba);

impl From<&GlobalLight> for DirectionalLight {
	fn from(GlobalLight(color): &GlobalLight) -> Self {
		DirectionalLight {
			shadows_enabled: false,
			illuminance: 100.,
			color: Color::from(*color),
			..default()
		}
	}
}

impl DerivableComponentFrom<GlobalLight> for DirectionalLight {
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;
}
