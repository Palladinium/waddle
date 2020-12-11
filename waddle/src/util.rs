use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type RcRC<T> = Rc<RefCell<T>>;
pub type WeakRC<T> = Weak<RefCell<T>>;
