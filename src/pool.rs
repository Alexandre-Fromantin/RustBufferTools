use std::{
    cell::RefCell,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    rc::Rc,
};

struct Pool<T> {
    stack: Vec<T>,
}

struct PoolRcRef<T> {
    rc: Rc<RefCell<Pool<T>>>,
}

impl<T: Default> Pool<T> {
    pub fn build(capacity: usize) -> PoolRcRef<T> {
        let mut stack = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            stack.push(T::default());
        }

        PoolRcRef {
            rc: Rc::new(RefCell::new(Self { stack })),
        }
    }
}

impl<T> PoolRcRef<T> {
    pub fn get(&self) -> Option<PoolGuard<T>> {
        let get_value = self.rc.borrow_mut().stack.pop()?;

        Some(PoolGuard {
            value: ManuallyDrop::new(get_value),
            pool_ref: PoolRcRef {
                rc: self.rc.clone(),
            },
        })
    }

    fn push(&self, value: T) {
        self.rc.borrow_mut().stack.push(value);
    }
}

struct PoolGuard<T> {
    value: ManuallyDrop<T>,
    pool_ref: PoolRcRef<T>,
}

impl<T> Drop for PoolGuard<T> {
    fn drop(&mut self) {
        let value = unsafe { ManuallyDrop::take(&mut self.value) };
        self.pool_ref.push(value);
    }
}

impl<T> Deref for PoolGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for PoolGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
