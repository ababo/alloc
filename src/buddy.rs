use core::mem::size_of;
use core::ptr::null;

use super::common::*;

#[inline]
fn align_addr_lower(addr: usize, order: u8) -> usize {
    addr >> order << order
}

#[inline]
fn align_addr_higher(addr: usize, order: u8) -> usize {
    align_addr_lower(addr + (1 << order) - 1, order)
}

struct Link {
    next: *mut Link,
    prev: *mut Link,
}

struct BuddyAlloc {
    min_order: u8,
    num_orders: u8,
    data: *mut *mut Link,
}

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

        let num_orders = order - min_order + 1;
        let heads_size = (num_orders as usize) * size_of::<*mut Link>();
        let map_len = (1 << (num_orders + 1)) - 2;
        let map_size = (map_len - 1) / 8 + 1;
        if to_addr - from_addr < size_of::<Link>() + heads_size + map_size {
            return Err(Error::NotEnoughSpaceForBookkeeping);
        }

        let mut alloc = BuddyAlloc {
            min_order: min_order,
            num_orders: num_orders,
            data: (from_addr + size_of::<Link>()) as *mut *mut Link,
        };

        alloc.set_map_bit(order, 0, true);
        alloc.set_map_bit(order, 1, false);
        *alloc.data = from_addr as *mut Link;
        **alloc.data = Link {
            next: null::<Link>() as *mut Link,
            prev: null::<Link>() as *mut Link,
        };

        loop {
            order -= 1;
            let ptr = align_addr_higher(addr, order);
            if ptr != from_addr {}

            let ptr = align_addr_lower(addr + size, order);
            if ptr != to_addr {}
        }

        Ok(alloc)
    }

    #[inline]
    fn map_bit(&self, order: u8, index: usize) -> bool {
        false
    }

    #[inline]
    fn set_map_bit(&mut self, order: u8, index: usize, value: bool) {}
}
