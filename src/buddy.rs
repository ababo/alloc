use core::intrinsics::{copy, write_bytes};
use core::mem::size_of;
use core::ptr::null;

use super::common::*;

struct Link {
    next: *mut Link,
    prev: *mut Link,
}

struct BuddyAlloc {
    min_order: u8,
    num_orders: u8,
    map_from_addr: usize,
    map_factor: usize,
    data: *mut *mut Link,
}

const USIZE_BITS: usize = usize::BITS as usize;

impl BuddyAlloc {
    pub unsafe fn new(
        min_order: u8,
        addr: usize,
        size: usize,
    ) -> Result<BuddyAlloc> {
        if (1 << min_order) < size_of::<Link>() {
            return Err(Error::MinOrderTooLow);
        }

        let mut order = (usize::BITS - size.leading_zeros() - 1) as u8;
        let mut from_addr;
        let mut to_addr;
        loop {
            from_addr = align_addr_higher(addr, order);
            to_addr = align_addr_lower(addr + size, order);
            if from_addr != to_addr {
                break; // Will be here on a first or a second iteration.
            }
            order -= 1;
        }

        let map_from_addr = align_addr_lower(addr, order);
        let map_to_addr = align_addr_higher(addr + size, order);

        let mut alloc = BuddyAlloc {
            min_order: min_order,
            num_orders: order - min_order + 1,
            map_from_addr: map_from_addr,
            map_factor: (map_to_addr - map_from_addr) >> order,
            data: (from_addr + size_of::<Link>()) as *mut *mut Link,
        };

        let heads_size = (alloc.num_orders as usize) * size_of::<*mut Link>();
        let map_len = alloc.map_order_offset(min_order - 1);
        let map_size = ((map_len - 1) / USIZE_BITS + 1) * size_of::<usize>();
        let data_size = heads_size + map_size;
        if to_addr - from_addr < size_of::<Link>() + data_size {
            return Err(Error::NotEnoughSpaceForBookkeeping);
        }

        write_bytes(alloc.data.offset(alloc.num_orders as isize), 0, map_size);
        *alloc.data = null::<*mut Link>() as *mut Link;
        alloc.add_free_block(order, from_addr);

        for order in order - 1..=min_order {
            *alloc.data.offset((order - alloc.min_order) as isize) =
                null::<*mut Link>() as *mut Link;

            let ptr = align_addr_higher(addr, order);
            if ptr != from_addr {
                alloc.add_free_block(order, ptr);
            }
            from_addr = ptr;

            let ptr = align_addr_lower(addr + size, order);
            if ptr != to_addr {
                alloc.add_free_block(order, ptr);
            }
            to_addr = ptr;
        }

        let data = alloc.alloc(size_order(data_size))? as *mut *mut Link;
        copy(data, alloc.data, data_size / size_of::<*mut Link>());
        alloc.data = data;

        Ok(alloc)
    }

    #[inline]
    fn map_order_offset(&self, order: u8) -> usize {
        let order_index = order - self.min_order;
        if order_index > 0 {
            1 + self.map_factor * ((1 << (order_index - 1)) - 1)
        } else {
            0 // The first bit is always for the highest order pair.
        }
    }

    #[inline]
    fn map_index(&self, order: u8, ptr: usize) -> usize {
        self.map_order_offset(order)
            + ((ptr - self.map_from_addr) >> (order + 1))
    }

    #[inline]
    unsafe fn map_bit(&self, order: u8, ptr: usize) -> bool {
        let index = self.map_index(order, ptr);
        let base = self.data.offset(self.num_orders as isize) as *const usize;
        let block = *base.offset((index / USIZE_BITS) as isize);
        let mask = 1 << (index % USIZE_BITS);
        block & mask != 0
    }

    #[inline]
    unsafe fn set_map_bit(&mut self, order: u8, ptr: usize, value: bool) {
        let index = self.map_index(order, ptr);
        let base = self.data.offset(self.num_orders as isize) as *mut usize;
        let block_ptr = base.offset((index / USIZE_BITS) as isize);
        let mask = 1 << (index % USIZE_BITS);
        if value {
            *block_ptr |= mask;
        } else {
            *block_ptr &= !mask;
        }
    }

    #[inline]
    unsafe fn add_free_block(&mut self, order: u8, ptr: usize) {
        let head = self.data.offset((order - self.min_order) as isize);
        let link = ptr as *mut Link;
        *link = Link {
            next: *head,
            prev: null::<Link>() as *mut Link,
        };
        if *head != null::<Link>() as *mut Link {
            (**head).prev = link;
        }
        *head = link;

        self.set_map_bit(order, ptr, true);
    }

    pub fn alloc(&mut self, order: u8) -> Result<usize> {
        Ok(0)
    }

    pub unsafe fn free(&mut self, order: u8, addr: usize) -> Result<()> {
        Ok(())
    }
}
