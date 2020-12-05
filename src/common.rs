pub enum Error {
    MinOrderTooLow,
    NotEnoughSpaceForBookkeeping,
}

pub type Result<T> = core::result::Result<T, Error>;
