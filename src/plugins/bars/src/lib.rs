pub mod components;
mod systems;
mod traits;

use bevy::{
	app::{App, FixedUpdate, Plugin, Update},
	ecs::{query::Without, schedule::IntoScheduleConfigs},
};
use common::{
	attributes::health::Health,
	traits::{
		after_plugin::AfterPlugin,
		handles_agents::HandlesAgents,
		handles_graphics::HandlesCameras,
		handles_physics::HandlesLife,
		ownership_relation::OwnershipRelation,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::bar::Bar;
use std::marker::PhantomData;
use systems::{bar::bar, render_bar::render_bar};

pub struct BarsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TAgents, TPhysics, TGraphics> BarsPlugin<(TAgents, TPhysics, TGraphics)>
where
	TAgents: ThreadSafe + HandlesAgents,
	TPhysics: ThreadSafe + SystemSetDefinition + HandlesLife,
	TGraphics: ThreadSafe + HandlesCameras,
{
	pub fn from_plugins(_: &TAgents, _: &TPhysics, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TAgents, TPhysics, TGraphics> Plugin for BarsPlugin<(TAgents, TPhysics, TGraphics)>
where
	TAgents: ThreadSafe + HandlesAgents,
	TPhysics: ThreadSafe + SystemSetDefinition + HandlesLife,
	TGraphics: ThreadSafe + HandlesCameras,
{
	fn build(&self, app: &mut App) {
		let update_life_bars = bar::<TPhysics::TAffectedComponent, Health, TGraphics::TCameraMut>;
		let render_life_bars = render_bar::<Health, TGraphics::TCameraMut>;

		app.manage_ownership::<Bar>(Update);
		app.add_systems(
			FixedUpdate,
			(
				Bar::add_to::<TAgents::TAgent<Without<Bar>>>,
				update_life_bars,
				render_life_bars,
			)
				.chain()
				.after_plugin(TPhysics::SYSTEMS),
		);
	}
}
