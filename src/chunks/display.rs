


// MARK: DisplayCh.

/// # WIP!
/// Structure that holds range of chunks displayed in the room.
#[derive(Clone, Debug, Default)]
pub struct DisplayChunks {
    pub total: u16,
    /// Oldest loaded chunk (should be lower number).
    pub start: u16,
    /// Youngest loaded chunk (should be bigger number).
    pub last: u16
}

impl DisplayChunks {
    /// Set complete range of loaded chunks.
    pub fn set_range(&mut self, start: u16, last: u16) {
        self.start = start;
        self.last = last;
    }
    /// Older chunk was loaded (`start` field).
    pub fn loaded_older(&mut self) {
        let new_val = self.start.saturating_sub(1);
        self.start = new_val;
    }
    
    /// New chunk was added to the front (`last` field).
    pub fn added_new_chunk(&mut self) {
        let new_val = self.last.saturating_add(1);
        self.last = new_val;
    }

    /// Deloded 1 or more old chunks.
    pub fn deloaded_old_chunks(&mut self, no_of_deloaded: u16) {
        self.start = no_of_deloaded;
    }
}

/// Chunk load cases.
#[derive(Clone, Debug)]
pub enum ChunkLoadCase {
    /// Nothing else to load: either all or nothing loaded.
    EverythingLoaded,
    /// Load first chunk.
    NothingLoaded,
    /// One chunk loaded, load next chunk.
    OneLoaded,
    /// Many chunks loaded, load next chunk.
    ManyLoaded,
}