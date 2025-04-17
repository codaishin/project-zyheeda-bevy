use common::tools::speed::Speed;
use std::time::Duration;

pub(crate) trait MinDistance {
	fn min_distance(speed: Speed, delta: Duration) -> f32;
}
