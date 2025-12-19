use std::{cell::RefCell, rc::Rc};

struct Pool<T> {
    stack: Vec<T>,
}

impl<T: Default> Pool<T> {
    pub fn new(capacity: usize) -> Rc<RefCell<Self>> {
        let mut stack = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            stack.push(T::default());
        }

        Rc::new(RefCell::new(Self { stack }))
    }

    pub fn get(pool: Rc<RefCell<Self>>) -> Option<PoolGuard<T>> {
        let get_value = pool.borrow_mut().stack.pop()?;

        Some(PoolGuard {
            value: get_value,
            pool: Rc::clone(&pool),
        })
    }
}

struct PoolGuard<T> {
    value: T,
    pool: Rc<RefCell<Pool<T>>>,
}
