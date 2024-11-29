mod components;
mod systems;

use bevy::prelude::*;
use common::{
	labels::Labels,
	traits::{handles_destruction::HandlesDestruction, handles_lifetime::HandlesLifetime},
};
use components::{destroy::Destroy, lifetime::Lifetime};
use std::time::Duration;
use systems::{destroy::destroy, destroy_dead::set_dead_to_be_destroyed};

pub struct LifeCyclesPlugin;

impl Plugin for LifeCyclesPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Labels::PROCESSING.label(), set_dead_to_be_destroyed)
			.add_systems(Labels::LAST.label(), destroy)
			.add_systems(Update, Lifetime::update::<Virtual>);
	}
}

impl HandlesLifetime for LifeCyclesPlugin {
	fn lifetime(duration: Duration) -> impl Bundle {
		Lifetime(duration)
	}
}

impl HandlesDestruction for LifeCyclesPlugin {
	type TDestroy = Destroy;
}
