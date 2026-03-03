use common::tools::vec_not_nan::Vec3NotNan;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Clone)]
pub struct MeshGridGraph(HashMap<Vec3NotNan, HashSet<Vec3NotNan>>);
