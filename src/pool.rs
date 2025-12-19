use std::rc::Rc;

struct Pool<T> {
    stack: Vec<T>,
}

impl<T: Default> Pool<T> {
    pub fn new(capacity: usize) -> Rc<Self> {
        let mut stack = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            stack.push(T::default());
        }

        Rc::new(Self { stack })
    }
}

struct PoolGuard<T> {
    value: T,
    pool: Rc<Pool<T>>,
}
