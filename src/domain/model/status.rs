#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryStatus {
    Pending,
    Unchanged,
    Modified,
    Renamed,
    Added,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    File,
    Directory,
}
