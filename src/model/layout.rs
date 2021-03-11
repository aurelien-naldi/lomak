use crate::helper::version::{Version, Versionned};
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Default, Debug)]
pub struct Layout {
    data: HashMap<usize, NodeLayoutInfo>,
    version: Version,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct NodeLayoutInfo {
    pub x: usize,
    pub y: usize,
    pub width: u8,
    pub height: u8,
}

impl Layout {
    pub fn set_bounding_box(&mut self, uid: usize, bb: NodeLayoutInfo) {
        self.data.insert(uid, bb);
    }
    pub fn get_bounding_box(&self, uid: usize) -> Option<&NodeLayoutInfo> {
        self.data.get(&uid)
    }
}

impl Versionned for Layout {
    fn version(&self) -> usize {
        self.version.version()
    }
}

impl fmt::Display for NodeLayoutInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{} [{},{}]", self.x, self.y, self.width, self.height)
    }
}
