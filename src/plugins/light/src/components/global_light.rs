use bevy::prelude::*;
use common::impl_savable_self_non_priority;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub(crate) struct GlobalLight(pub(crate) Srgba);

impl GlobalLight {
	pub(crate) fn light(&self) -> DirectionalLight {
		DirectionalLight {
			shadows_enabled: false,
			illuminance: 100.,
			color: Color::from(self.0),
			..default()
		}
	}
}

impl_savable_self_non_priority!(GlobalLight);
