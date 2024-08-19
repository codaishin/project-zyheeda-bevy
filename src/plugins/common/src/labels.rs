use bevy::app::{PostUpdate, PreUpdate, Update};

pub struct Labels;

impl Labels {
	pub const INSTANTIATION: PreUpdate = PreUpdate;
	pub const PROCESSING: Update = Update;
	pub const PROPAGATION: PostUpdate = PostUpdate;
}
