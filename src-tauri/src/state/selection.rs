use std::sync::RwLock;

pub struct SelectionState {
    inner: RwLock<SelectionInner>,
}

#[derive(Default)]
pub struct SelectionInner {
    pub selected_channels: Vec<usize>,
    pub active_block: usize,
}

impl Default for SelectionState {
    fn default() -> Self {
        SelectionState {
            inner: RwLock::new(SelectionInner::default()),
        }
    }
}

impl SelectionState {
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, SelectionInner> {
        self.inner.read().unwrap()
    }
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, SelectionInner> {
        self.inner.write().unwrap()
    }
}
