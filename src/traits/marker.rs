pub mod hand_gun;
pub mod sword;

use crate::markers::meta::MarkerMeta;

pub trait GetMarkerMeta {
	fn marker() -> MarkerMeta;
}
