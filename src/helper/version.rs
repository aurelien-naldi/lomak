//! Define and maintain versioned data structures

use std::cell::Cell;

/// A versioned data structure has a ```version``` method which gives a new version number if
/// the content of the structure has changed since its last call.
pub trait Versionned {
    fn version(&self) -> usize;
}

/// Store and update the current version number.
/// Use this struct as a field to implement the ```Versionned``` trait.
#[derive(Default, Debug, Clone)]
pub struct Version {
    inner: Cell<InnerVersion>,
}

#[derive(Default, Debug, Copy, Clone)]
struct InnerVersion {
    changed: bool,
    version: usize,
}

impl Version {
    pub fn change(&mut self) {
        let mut i = self.inner.get();
        if !i.changed {
            i.changed = true;
            self.inner.set(i);
        }
    }

    pub fn version(&self) -> usize {
        let mut i = self.inner.get();
        if i.changed {
            i.version += 1;
            self.inner.set(InnerVersion {
                changed: false,
                version: i.version,
            });
        }
        i.version
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn version_check() {
        let mut v = Version::default();
        assert_eq!(v.version(), 0);

        v.change();
        assert_eq!(v.version(), 1);

        v.change();
        v.change();
        assert_eq!(v.version(), 2);
    }
}
