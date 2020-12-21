use crate::helper::version::{Version, Versionned};
use std::collections::HashMap;

pub struct Layout {
    data: HashMap<usize, NodeLayoutInfo>,
    version: Version,
}

pub struct NodeLayoutInfo {
    x: usize,
    y: usize,
    width: u8,
    height: u8,
}

impl Versionned for Layout {
    fn version(&self) -> usize {
        self.version.version()
    }
}