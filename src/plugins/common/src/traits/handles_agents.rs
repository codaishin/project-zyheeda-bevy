use bevy::ecs::component::Component;

pub trait HandlesAgents {
	type TAgent: Component;
}
