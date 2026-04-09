use std::sync::RwLock;

pub struct SelectionState {
    selection: RwLock<Selection>,
}

#[derive(Default)]
pub struct Selection {
    pub selected_channels: Vec<usize>,
    pub active_block: usize,
}

impl Default for SelectionState {
    fn default() -> Self {
        SelectionState {
            selection: RwLock::new(Selection::default()),
        }
    }
}

impl SelectionState {
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, Selection> {
        self.selection.read().unwrap()
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, Selection> {
        self.selection.write().unwrap()
    }
}
