mod components;
mod systems;

use bevy::prelude::*;
use common::{labels::Labels, traits::handles_lifetime::HandlesLifetime};
use components::lifetime::Lifetime;
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
	type TLifetime = Lifetime;
}
