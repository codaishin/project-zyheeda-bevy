mod components;
mod systems;

use bevy::prelude::*;
use common::traits::{
	handles_destruction::HandlesDestruction,
	handles_life::HandlesLife,
	handles_lifetime::HandlesLifetime,
	handles_saving::HandlesSaving,
	thread_safe::ThreadSafe,
};
use components::{destroy::Destroy, life::Life, lifetime::Lifetime};
use std::{marker::PhantomData, time::Duration};
use systems::{destroy::destroy, destroy_dead::set_dead_to_be_destroyed};

pub struct LifeCyclesPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame> LifeCyclesPlugin<TSaveGame>
where
	TSaveGame: ThreadSafe + HandlesSaving,
{
	pub fn from_plugin(_: &TSaveGame) -> Self {
		Self(PhantomData)
	}
}

impl<TSaveGame> Plugin for LifeCyclesPlugin<TSaveGame>
where
	TSaveGame: ThreadSafe + HandlesSaving,
{
	fn build(&self, app: &mut App) {
		TSaveGame::register_savable_component::<Lifetime>(app);

		app.add_systems(Update, set_dead_to_be_destroyed)
			.add_systems(PostUpdate, destroy)
			.add_systems(Update, Lifetime::update::<Virtual>);
	}
}

impl<TDependencies> HandlesLifetime for LifeCyclesPlugin<TDependencies> {
	fn lifetime(duration: Duration) -> impl Bundle {
		Lifetime(duration)
	}
}

impl<TDependencies> HandlesDestruction for LifeCyclesPlugin<TDependencies> {
	type TDestroy = Destroy;
}

impl<TDependencies> HandlesLife for LifeCyclesPlugin<TDependencies> {
	type TLife = Life;
}
