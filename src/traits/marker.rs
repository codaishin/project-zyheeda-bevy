use crate::markers::{
	functions::{insert_hand_marker_fn, remove_hand_marker_fn},
	meta::MarkerMeta,
	Left,
	Right,
};

pub trait GetMarkerHandMarkerMeta {
	fn hand_markers() -> MarkerMeta;
}

impl<T: Send + Sync + 'static> GetMarkerHandMarkerMeta for T {
	fn hand_markers() -> MarkerMeta {
		MarkerMeta {
			insert_fn: insert_hand_marker_fn::<(T, Left), (T, Right)>,
			remove_fn: remove_hand_marker_fn::<(T, Left), (T, Right)>,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn insert_fn() {
		let markers = u32::hand_markers();

		assert_eq!(
			insert_hand_marker_fn::<(u32, Left), (u32, Right)> as usize,
			markers.insert_fn as usize,
		)
	}

	#[test]
	fn remove_fn() {
		let markers = u32::hand_markers();

		assert_eq!(
			remove_hand_marker_fn::<(u32, Left), (u32, Right)> as usize,
			markers.remove_fn as usize,
		)
	}
}
