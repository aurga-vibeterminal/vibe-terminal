use std::collections::VecDeque;

const RAW_HISTORY_MAX_BYTES: usize = 1024 * 1024;
const RAW_HISTORY_CHUNK_BYTES: usize = 128 * 1024;

struct RawHistoryChunk {
    start_offset: u64,
    data: Vec<u8>,
}

pub struct RawHistoryBuffer {
    chunks: VecDeque<RawHistoryChunk>,
    start_offset: u64,
    end_offset: u64,
    retained_bytes: usize,
    max_bytes: usize,
}

impl RawHistoryBuffer {
    pub(super) fn new_default() -> Self {
        Self::new(RAW_HISTORY_MAX_BYTES)
    }

    fn new(max_bytes: usize) -> Self {
        Self {
            chunks: VecDeque::new(),
            start_offset: 0,
            end_offset: 0,
            retained_bytes: 0,
            max_bytes: max_bytes.max(1),
        }
    }

    pub(super) fn start_offset(&self) -> u64 {
        self.start_offset
    }

    pub(super) fn end_offset(&self) -> u64 {
        self.end_offset
    }

    pub(super) fn append(&mut self, data: &[u8]) -> u64 {
        let start_offset = self.end_offset;
        if data.is_empty() {
            return start_offset;
        }

        let mut written = 0;
        while written < data.len() {
            let next_len = (data.len() - written).min(RAW_HISTORY_CHUNK_BYTES);
            let chunk = data[written..written + next_len].to_vec();
            self.chunks.push_back(RawHistoryChunk {
                start_offset: self.end_offset,
                data: chunk,
            });
            self.end_offset += next_len as u64;
            self.retained_bytes += next_len;
            written += next_len;
        }

        while self.retained_bytes > self.max_bytes {
            let Some(chunk) = self.chunks.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(chunk.data.len());
            self.start_offset = chunk.start_offset + chunk.data.len() as u64;
        }

        if self.chunks.is_empty() {
            self.start_offset = self.end_offset;
        }

        start_offset
    }

    pub(super) fn read_before(&self, before_offset: u64, max_bytes: usize) -> (u64, Vec<u8>) {
        if max_bytes == 0 || self.start_offset >= self.end_offset {
            return (before_offset.min(self.end_offset), Vec::new());
        }

        let clamped_end = before_offset.min(self.end_offset);
        if clamped_end <= self.start_offset {
            return (self.start_offset, Vec::new());
        }

        let clamped_start = clamped_end
            .saturating_sub(max_bytes as u64)
            .max(self.start_offset);
        let mut data = Vec::with_capacity((clamped_end - clamped_start) as usize);

        for chunk in &self.chunks {
            let chunk_end = chunk.start_offset + chunk.data.len() as u64;
            if chunk_end <= clamped_start {
                continue;
            }
            if chunk.start_offset >= clamped_end {
                break;
            }

            let slice_start = clamped_start.saturating_sub(chunk.start_offset) as usize;
            let slice_end = (clamped_end.min(chunk_end) - chunk.start_offset) as usize;
            data.extend_from_slice(&chunk.data[slice_start..slice_end]);
        }

        (clamped_start, data)
    }

    pub(super) fn snapshot(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.retained_bytes);
        for chunk in &self.chunks {
            data.extend_from_slice(&chunk.data);
        }
        data
    }

    /// Drop all retained bytes while keeping the stream offset space
    /// monotonic: the window becomes empty at the current end offset, so
    /// subsequent appends continue from `end_offset` as if nothing was
    /// retained before. Used by the UART "clear screen" command.
    pub(super) fn clear_retained(&mut self) {
        self.chunks.clear();
        self.retained_bytes = 0;
        self.start_offset = self.end_offset;
    }
}
