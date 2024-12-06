use bevy::prelude::Component;

pub trait HandlesPlayer {
	type TPlayer: Component;
}
