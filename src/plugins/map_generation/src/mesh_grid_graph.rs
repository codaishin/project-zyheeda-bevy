use common::tools::vec_not_nan::VecNotNan;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Clone)]
pub struct MeshGridGraph(HashMap<VecNotNan<3>, HashSet<VecNotNan<3>>>);
