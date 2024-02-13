mod components;
mod systems;
mod traits;

use bevy::app::{App, Plugin, PostUpdate, Update};
use common::components::{DealsDamage, Health};
use systems::{
	collision::{destroy_on_collision::destroy_on_collision, interaction::collision_interaction},
	destroy::destroy,
};

pub struct InteractionsPlugin;

impl Plugin for InteractionsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, collision_interaction::<DealsDamage, Health>)
			.add_systems(Update, destroy)
			.add_systems(PostUpdate, destroy_on_collision);
	}
}
