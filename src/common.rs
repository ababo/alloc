pub enum Error {
    MinOrderTooLow,
    NotEnoughSpaceForBookkeeping,
}

pub type Result<T> = core::result::Result<T, Error>;

#[inline]
pub fn align_addr_lower(addr: usize, order: u8) -> usize {
    addr >> order << order
}

#[inline]
pub fn align_addr_higher(addr: usize, order: u8) -> usize {
    align_addr_lower(addr + (1 << order) - 1, order)
}

#[inline]
pub fn size_order(size: usize) -> u8 {
    let order = (usize::BITS - size.leading_zeros() - 1) as u8;
    if size == (1 << order) {
        order
    } else {
        order + 1
    }
}
