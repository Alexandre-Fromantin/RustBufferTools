use std::{
    cell::RefCell,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    rc::Rc,
};

struct Pool<T> {
    stack: Vec<T>,
}

#[derive(Clone)]
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
    pub fn acquire(&self) -> Option<PoolGuard<T>> {
        let acquire_value = self.rc.borrow_mut().stack.pop()?;

        Some(PoolGuard {
            value: ManuallyDrop::new(acquire_value),
            pool_ref: PoolRcRef {
                rc: self.rc.clone(),
            },
        })
    }

    fn release(&self, value: T) {
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
        self.pool_ref.release(value);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack() {
        let pool: PoolRcRef<u32> = Pool::build(150);
        for _ in 0..150 {
            pool.acquire();
        }

        let mut guard_list = Vec::new();
        for _ in 0..150 {
            guard_list.push(pool.acquire());
        }

        assert!(pool.acquire().is_none());
    }

    #[test]
    fn test_stack_v2() {
        let pool: PoolRcRef<u32> = Pool::build(150);
        for _ in 0..150 {
            pool.acquire();
        }

        let mut guard_list = Vec::new();
        for _ in 0..148 {
            guard_list.push(pool.acquire());
        }

        let value_1 = pool.acquire();
        assert!(value_1.is_some());

        let value_2 = pool.acquire();
        assert!(value_2.is_some());

        assert!(pool.acquire().is_none());
    }

    #[test]
    fn test_stack_v3() {
        let pool: PoolRcRef<u32> = Pool::build(150);

        let mut temp = pool.acquire().unwrap();
        *temp = 99;
        drop(temp);

        for _ in 0..1000 {
            let value = pool.acquire().unwrap();
            assert_eq!(*value, 99);
        }
    }

    #[test]
    fn test_stack_clone() {
        let pool: PoolRcRef<u32> = Pool::build(150);
        for _ in 0..150 {
            let mut value = pool.acquire().unwrap();
            *value = 5
        }

        let pool_2 = pool.clone();

        for _ in 0..150 {
            let value = pool_2.acquire().unwrap();
            assert_eq!(*value, 5)
        }
    }
}
