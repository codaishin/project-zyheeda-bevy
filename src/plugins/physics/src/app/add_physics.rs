use crate::{
	CollisionSystems,
	PhysicsSystems,
	components::ongoing_effects::OngoingEffects,
	systems::interactions::act_on::ActOnSystem,
	traits::act_on::ActOn,
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::traits::{
	delta::Delta,
	handles_saving::{HandlesSaving, SavableComponent},
};

pub(crate) trait AddPhysics {
	fn add_physics<TActor, TTarget, TSaveGame>(&mut self) -> &mut Self
	where
		TActor: ActOn<TTarget> + Component<Mutability = Mutable> + SavableComponent,
		TTarget: Component<Mutability = Mutable> + SavableComponent,
		TSaveGame: HandlesSaving;
}

impl AddPhysics for App {
	fn add_physics<TActor, TTarget, TSaveGame>(&mut self) -> &mut Self
	where
		TActor: ActOn<TTarget> + Component<Mutability = Mutable> + SavableComponent,
		TTarget: Component<Mutability = Mutable> + SavableComponent,
		TSaveGame: HandlesSaving,
	{
		TSaveGame::register_savable_component::<TActor>(self);
		TSaveGame::register_savable_component::<TTarget>(self);
		TSaveGame::register_savable_component::<OngoingEffects<TActor, TTarget>>(self);

		self.register_required_components::<TActor, OngoingEffects<TActor, TTarget>>();
		self.add_systems(
			Update,
			Update::delta
				.pipe(TActor::act_on::<TTarget>)
				.in_set(PhysicsSystems)
				.after(CollisionSystems),
		)
	}
}
