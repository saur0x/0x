use crate::device::Device;

use super::{Byte, Word};

pub struct MemoryMapper {
    pub regions: Vec<Region>,
}

pub struct Region {
    pub device: Box<dyn Device>,
    pub start: Word,
    pub end: Word,
    pub remap: bool,
}

#[allow(dead_code)]
impl MemoryMapper {
    pub fn new() -> MemoryMapper {
        MemoryMapper {
            regions: Vec::new(),
        }
    }

    fn find_region(&self, addr: Word) -> usize {
        for (i, region) in self.regions.iter().enumerate() {
            if region.start <= addr && addr < region.end {
                return i;
            }
        }
        panic!("[MEMORY MAPPER] No such region: '0x{:08X}'", addr);
    }

    fn get_region_and_addr(&self, addr: Word) -> (usize, Word) {
        let region_index = self.find_region(addr);
        let final_addr = if self.regions[region_index].remap {
            addr - self.regions[region_index].start
        } else {
            addr
        };

        (region_index, final_addr)
    }

    pub fn get_word(&self, addr: Word) -> Word {
        let (region_index, final_addr) = self.get_region_and_addr(addr);

        self.regions[region_index].device.get_word(final_addr)
    }

    pub fn get_byte(&self, addr: Word) -> Byte {
        let (region_index, final_addr) = self.get_region_and_addr(addr);

        self.regions[region_index].device.get_byte(final_addr)
    }

    pub fn set_word(&mut self, addr: Word, value: Word) {
        let (region_index, final_addr) = self.get_region_and_addr(addr);

        self.regions[region_index]
            .device
            .set_word(final_addr, value);
    }

    pub fn set_byte(&mut self, addr: Word, value: Byte) {
        let (region_index, final_addr) = self.get_region_and_addr(addr);

        self.regions[region_index]
            .device
            .set_byte(final_addr, value);
    }

    pub fn map(&mut self, device: Box<dyn Device>, start: Word, end: Word, remap: bool)
    // -> Box<dyn Fn(&mut MemoryMapper)> {
    {
        let region = Region {
            device,
            start,
            end,
            remap,
        };

        self.regions.insert(0, region);

        /*
        Box::new(move |this: &mut MemoryMapper| {
            for (i, r) in this.regions.iter().enumerate() {
                if *r == region {
                    this.regions.remove(i);
                    break;
                }
            }
        })
        */
    }
}
