// ## Map
// Role: core/ring_buffer — 手機即時音訊 (Oboe/AudioRecord) 與感測器串流之純記憶體固定容量環形緩衝區 (Ring Buffer) 微積木。
// Inputs: PCM16 取樣點、取樣切片、緩衝區容量。
// Outputs: 讀寫成功狀態、彈出樣本、實際讀寫長度、可用讀寫容量。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、有界容量守衛（最大 131,072 取樣點）、控制流巢狀深度 <= 2、純標準庫 100% 零依賴。

const MAX_RING_CAPACITY: usize = 131_072; // 約 48kHz 下 2.7 秒緩衝

pub struct RingBufferPcm16 {
    storage: Vec<i16>,
    read_pos: usize,
    write_pos: usize,
    capacity: usize,
}

impl RingBufferPcm16 {
    /// @outputs: 2
    /// @domain: capacity [2..131072]
    pub fn with_capacity(capacity: usize) -> Result<Self, &'static str> {
        if capacity < 2 || capacity > MAX_RING_CAPACITY {
            return Err("容量超出有界範圍 [2..131072]");
        }
        // 分配 capacity + 1 格以區分滿與空
        let alloc_size = capacity + 1;
        Ok(Self {
            storage: vec![0i16; alloc_size],
            read_pos: 0,
            write_pos: 0,
            capacity: alloc_size,
        })
    }

    /// @outputs: 1
    pub fn available_read(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.capacity - self.read_pos + self.write_pos
        }
    }

    /// @outputs: 1
    pub fn available_write(&self) -> usize {
        (self.capacity - 1) - self.available_read()
    }

    /// @outputs: 2
    pub fn push(&mut self, sample: i16) -> bool {
        if self.available_write() == 0 {
            return false;
        }
        self.storage[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % self.capacity;
        true
    }

    /// @outputs: 2
    pub fn pop(&mut self) -> Option<i16> {
        if self.available_read() == 0 {
            return None;
        }
        let sample = self.storage[self.read_pos];
        self.read_pos = (self.read_pos + 1) % self.capacity;
        Some(sample)
    }

    /// @outputs: 1
    pub fn write_chunk(&mut self, samples: &[i16]) -> usize {
        let max_write = self.available_write();
        let to_write = if samples.len() < max_write { samples.len() } else { max_write };
        for i in 0..to_write {
            self.storage[self.write_pos] = samples[i];
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
        to_write
    }

    /// @outputs: 1
    pub fn read_chunk(&mut self, out: &mut [i16]) -> usize {
        let max_read = self.available_read();
        let to_read = if out.len() < max_read { out.len() } else { max_read };
        for i in 0..to_read {
            out[i] = self.storage[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        to_read
    }
}
