//! Sub-allocator for PPMd model memory management.
//!
//! Based on Dmitry Shkarin's implementation.

/// Number of unit size classes.
const N1: usize = 4;
const N2: usize = 4;
const N3: usize = 4;
const N4: usize = (128 + 3 - N1 - 2 * N2 - 3 * N3) / 4;
const N_INDEXES: usize = N1 + N2 + N3 + N4;

/// Unit size in bytes (12 bytes - size of a context).
const UNIT_SIZE: usize = 12;

/// Memory block header.
struct MemBlock {
    stamp: u16,
    nu: u16,
    next: u32,
    prev: u32,
}

/// Free list node.
struct FreeNode {
    next: u32,
}

/// Sub-allocator for PPMd contexts.
pub struct SubAllocator {
    /// Heap memory.
    heap: Vec<u8>,
    /// Start of text area (grows up).
    text_ptr: usize,
    /// Start of units area.
    units_start: usize,
    /// Low unit pointer (grows up).
    lo_unit: usize,
    /// High unit pointer (grows down).
    hi_unit: usize,
    /// End of heap.
    heap_end: usize,
    /// Free lists for each size class.
    free_list: [u32; N_INDEXES],
    /// Index to number of units mapping.
    idx2units: [u8; N_INDEXES],
    /// Number of units to index mapping.
    units2idx: [u8; 128],
    /// Glue counter.
    glue_count: u8,
}

impl SubAllocator {
    /// Create a new sub-allocator with the given size in MB.
    pub fn new(size_mb: usize) -> Self {
        let size = size_mb * 1024 * 1024;
        let mut sa = Self {
            heap: vec![0u8; size],
            text_ptr: 0,
            units_start: 0,
            lo_unit: 0,
            hi_unit: 0,
            heap_end: size,
            free_list: [0; N_INDEXES],
            idx2units: [0; N_INDEXES],
            units2idx: [0; 128],
            glue_count: 0,
        };
        sa.init_tables();
        sa.init();
        sa
    }

    /// Initialize the index/units mapping tables.
    fn init_tables(&mut self) {
        let mut i = 0usize;
        let mut k: u8;

        // First N1 indices: k=1,2,3,4
        for j in 0..N1 {
            self.idx2units[i] = (j + 1) as u8;
            i += 1;
        }

        // Next N2 indices: k starts at 6 (skips 5), steps by 2: 6,8,10,12
        k = (N1 as u8) + 2; // k = 6
        for _ in 0..N2 {
            self.idx2units[i] = k;
            i += 1;
            k += 2;
        }

        // Next N3 indices: k starts at k+1, steps by 3: 13,16,19,22
        k += 1; // 12 + 1 = 13
        for _ in 0..N3 {
            self.idx2units[i] = k;
            i += 1;
            k += 3;
        }

        // Remaining N4 indices: k starts at k+1, steps by 4: 23,27,31,...
        k += 1; // 22 + 1 = 23
        for _ in 0..N4 {
            self.idx2units[i] = k;
            i += 1;
            k += 4;
        }

        // Build reverse mapping
        let mut idx = 0u8;
        for j in 0..128 {
            while idx < N_INDEXES as u8 && (self.idx2units[idx as usize] as usize) < j + 1 {
                idx += 1;
            }
            // Clamp to valid index range
            self.units2idx[j] = idx.min(N_INDEXES as u8 - 1);
        }
    }

    /// Initialize/reset the allocator.
    pub fn init(&mut self) {
        self.glue_count = 0;
        for i in 0..N_INDEXES {
            self.free_list[i] = 0;
        }

        // unrar uses 1/8 of memory for text area
        // Size2 = (SubAllocatorSize/8/UNIT_SIZE*7) * UNIT_SIZE = 7/8 of memory
        // Size1 = SubAllocatorSize - Size2 = 1/8 of memory
        let text_size = self.heap_end / 8;
        // Align to UNIT_SIZE
        let text_size = (text_size / UNIT_SIZE) * UNIT_SIZE + UNIT_SIZE;
        self.text_ptr = 0;
        self.units_start = text_size;
        self.lo_unit = self.units_start;
        self.hi_unit = self.heap_end;
    }

    /// Resize the allocator if the new size is different.
    /// Reuses existing buffer if size matches, avoiding reallocation.
    pub fn resize(&mut self, size_mb: usize) {
        let size = size_mb * 1024 * 1024;
        if self.heap.len() != size {
            self.heap = vec![0u8; size];
            self.heap_end = size;
            self.init_tables();
        }
        self.init();
    }

    /// Get heap end pointer.
    pub fn heap_end(&self) -> usize {
        self.heap_end
    }

    /// Get text pointer.
    pub fn text_ptr(&self) -> usize {
        self.text_ptr
    }

    /// Allocate a context (1 unit).
    pub fn alloc_context(&mut self) -> Option<usize> {
        if self.hi_unit != self.lo_unit {
            self.hi_unit -= UNIT_SIZE;
            return Some(self.hi_unit);
        }
        self.alloc_units_rare(0)
    }

    /// Allocate units.
    pub fn alloc_units(&mut self, nu: usize) -> Option<usize> {
        if nu == 0 || nu > 128 {
            return None;
        }
        // Clamp to valid index range (units2idx is only 128 entries)
        let lookup_nu = if nu >= 128 { 127 } else { nu };
        let idx = self.units2idx[lookup_nu] as usize;

        #[cfg(test)]
        if nu == 128 {
            eprintln!(
                "[ALLOC] alloc_units(128): lookup_nu={} idx={} idx2units[idx]={}",
                lookup_nu, idx, self.idx2units[idx]
            );
        }

        // Try free list first
        if self.free_list[idx] != 0 {
            let ptr = self.remove_node(idx)?;

            // Debug trace
            #[cfg(test)]
            if (1024..2560).contains(&ptr) {
                eprintln!("[ALLOC] WARNING: alloc_units({}) from FREE LIST returned {} which overlaps root stats [1024, 2560)", nu, ptr);
            }

            return Some(ptr);
        }

        // Try to allocate from lo_unit
        let units_needed = self.idx2units[idx] as usize;
        let bytes_needed = units_needed * UNIT_SIZE;

        #[cfg(test)]
        if nu == 128 {
            eprintln!(
                "[ALLOC] alloc_units(128): units_needed={} bytes_needed={} lo_unit={}",
                units_needed, bytes_needed, self.lo_unit
            );
        }

        if self.lo_unit + bytes_needed <= self.hi_unit {
            let ptr = self.lo_unit;
            self.lo_unit += bytes_needed;

            // Debug trace
            #[cfg(test)]
            if (1024..2560).contains(&ptr) {
                eprintln!("[ALLOC] WARNING: alloc_units({}) from LO_UNIT returned {} which overlaps root stats [1024, 2560)", nu, ptr);
            }

            return Some(ptr);
        }

        let ptr = self.alloc_units_rare(idx)?;

        // Debug trace
        #[cfg(test)]
        if (1024..2560).contains(&ptr) {
            eprintln!("[ALLOC] WARNING: alloc_units({}) from RARE returned {} which overlaps root stats [1024, 2560)", nu, ptr);
        }

        Some(ptr)
    }

    /// Allocate from rare path (try larger free lists).
    fn alloc_units_rare(&mut self, idx: usize) -> Option<usize> {
        // Try larger free lists
        for i in (idx + 1)..N_INDEXES {
            if self.free_list[i] != 0 {
                if let Some(ptr) = self.remove_node(i) {
                    // Split the block
                    self.split_block(ptr, i, idx);
                    return Some(ptr);
                }
            }
        }

        // Try gluing free blocks
        self.glue_count += 1;
        if self.glue_count > 13 {
            self.glue_free_blocks();
            self.glue_count = 0;
        }

        None
    }

    /// Remove a node from free list.
    fn remove_node(&mut self, idx: usize) -> Option<usize> {
        let ptr = self.free_list[idx] as usize;
        if ptr == 0 {
            return None;
        }

        // Read next pointer from the node
        let next = self.read_u32(ptr);
        self.free_list[idx] = next;
        Some(ptr)
    }

    /// Insert a node into free list.
    fn insert_node(&mut self, ptr: usize, idx: usize) {
        let old_head = self.free_list[idx];
        self.write_u32(ptr, old_head);
        self.free_list[idx] = ptr as u32;
    }

    /// Split a block when a smaller one is needed.
    fn split_block(&mut self, ptr: usize, old_idx: usize, new_idx: usize) {
        let old_units = self.idx2units[old_idx] as usize;
        let new_units = self.idx2units[new_idx] as usize;
        let diff = old_units - new_units;

        if diff > 0 {
            let new_ptr = ptr + new_units * UNIT_SIZE;
            let diff_idx = self.units2idx[diff.saturating_sub(1)] as usize;
            self.insert_node(new_ptr, diff_idx);
        }
    }

    /// Glue free blocks together.
    fn glue_free_blocks(&mut self) {
        // Simplified: just clear free lists and hope for the best
        // Full implementation would merge adjacent blocks
    }

    /// Free units.
    pub fn free_units(&mut self, ptr: usize, nu: usize) {
        if nu >= 128 {
            return;
        }
        let idx = self.units2idx[nu] as usize;
        self.insert_node(ptr, idx);
    }

    /// Read a byte from heap.
    #[inline]
    pub fn read_byte(&self, offset: usize) -> u8 {
        // SAFETY: PPMd validates offsets against heap_end at context level
        debug_assert!(offset < self.heap.len());
        // SAFETY: bounds checked in debug, validated at higher level in release
        unsafe { *self.heap.get_unchecked(offset) }
    }

    /// Write a byte to heap.
    #[inline]
    pub fn write_byte(&mut self, offset: usize, val: u8) {
        debug_assert!(offset < self.heap.len());
        // SAFETY: bounds checked in debug, validated at higher level in release
        unsafe { *self.heap.get_unchecked_mut(offset) = val };
    }

    /// Read a u32 from heap.
    #[inline]
    pub fn read_u32(&self, offset: usize) -> u32 {
        debug_assert!(offset + 4 <= self.heap.len());
        // SAFETY: bounds checked in debug, validated at higher level in release
        unsafe {
            let ptr = self.heap.as_ptr().add(offset);
            u32::from_le_bytes([*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)])
        }
    }

    /// Write a u32 to heap.
    #[inline]
    pub fn write_u32(&mut self, offset: usize, val: u32) {
        debug_assert!(offset + 4 <= self.heap.len());
        // SAFETY: bounds checked in debug, validated at higher level in release
        unsafe {
            let ptr = self.heap.as_mut_ptr().add(offset);
            let bytes = val.to_le_bytes();
            *ptr = bytes[0];
            *ptr.add(1) = bytes[1];
            *ptr.add(2) = bytes[2];
            *ptr.add(3) = bytes[3];
        }
    }

    /// Read a u16 from heap.
    #[inline]
    pub fn read_u16(&self, offset: usize) -> u16 {
        debug_assert!(offset + 2 <= self.heap.len());
        // SAFETY: bounds checked in debug, validated at higher level in release
        unsafe {
            let ptr = self.heap.as_ptr().add(offset);
            u16::from_le_bytes([*ptr, *ptr.add(1)])
        }
    }

    /// Write a u16 to heap.
    #[inline]
    pub fn write_u16(&mut self, offset: usize, val: u16) {
        debug_assert!(offset + 2 <= self.heap.len());
        // SAFETY: bounds checked in debug, validated at higher level in release
        unsafe {
            let ptr = self.heap.as_mut_ptr().add(offset);
            let bytes = val.to_le_bytes();
            *ptr = bytes[0];
            *ptr.add(1) = bytes[1];
        }
    }

    /// Get text pointer.
    pub fn get_text_ptr(&self) -> usize {
        self.text_ptr
    }

    /// Get units start.
    pub fn get_units_start(&self) -> usize {
        self.units_start
    }

    /// Advance text pointer by 1.
    pub fn advance_text_ptr(&mut self) {
        if self.text_ptr < self.units_start {
            self.text_ptr += 1;
        }
    }

    /// Retreat text pointer by 1.
    pub fn retreat_text_ptr(&mut self) {
        if self.text_ptr > 0 {
            self.text_ptr -= 1;
        }
    }

    /// Expand units allocation.
    /// Returns new pointer or None if allocation fails.
    pub fn expand_units(&mut self, old_ptr: usize, old_nu: usize) -> Option<usize> {
        let old_idx = if old_nu >= 128 {
            N_INDEXES - 1
        } else {
            self.units2idx[old_nu] as usize
        };
        let new_nu = old_nu + 1;
        let new_idx = if new_nu >= 128 {
            N_INDEXES - 1
        } else {
            self.units2idx[new_nu] as usize
        };

        if old_idx == new_idx {
            // Same size class, no need to reallocate
            return Some(old_ptr);
        }

        // Need to allocate new block and copy
        let new_ptr = self.alloc_units(new_nu)?;

        // Copy old data
        let copy_size = self.idx2units[old_idx] as usize * UNIT_SIZE;
        for i in 0..copy_size {
            let byte = self.read_byte(old_ptr + i);
            self.write_byte(new_ptr + i, byte);
        }

        // Free old block
        self.free_units(old_ptr, old_nu);

        Some(new_ptr)
    }
}
