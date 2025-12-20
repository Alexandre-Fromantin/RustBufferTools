use std::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    rc::Rc,
};

struct PoolConfig {
    min_allocation: usize,
    max_allocation: usize,
}

struct Pool<T> {
    config: PoolConfig,
    free_stack: Vec<T>,
    nb_allocation: usize,
}

#[derive(Clone)]
struct PoolRef<T> {
    pool_ref: Rc<UnsafeCell<Pool<T>>>,
}

impl<T: Default> Pool<T> {
    pub fn build(config: PoolConfig) -> PoolRef<T> {
        let mut free_stack = Vec::with_capacity(config.min_allocation);

        for _ in 0..config.min_allocation {
            free_stack.push(T::default());
        }

        PoolRef {
            pool_ref: Rc::new(UnsafeCell::new(Self {
                free_stack,
                nb_allocation: config.min_allocation,
                config,
            })),
        }
    }
}

impl<T: Default> PoolRef<T> {
    pub fn acquire(&self) -> Option<PoolGuard<T>> {
        let pool = unsafe { &mut *self.pool_ref.get() };

        let acquire_value = pool.free_stack.pop().or_else(|| {
            if pool.nb_allocation == pool.config.max_allocation {
                return None;
            }
            pool.nb_allocation += 1;
            Some(T::default())
        })?;

        Some(PoolGuard {
            value: ManuallyDrop::new(acquire_value),
            pool_ref: PoolRef {
                pool_ref: self.pool_ref.clone(),
            },
        })
    }

    fn release(&self, value: T) {
        let pool = unsafe { &mut *self.pool_ref.get() };

        if pool.free_stack.len() == pool.free_stack.capacity() {
            //over-allocation
            return;
        }

        pool.free_stack.push(value);
    }
}

struct PoolGuard<T: Default> {
    value: ManuallyDrop<T>,
    pool_ref: PoolRef<T>,
}

impl<T: Default> Drop for PoolGuard<T> {
    fn drop(&mut self) {
        let value = unsafe { ManuallyDrop::take(&mut self.value) };
        self.pool_ref.release(value);
    }
}

impl<T: Default> Deref for PoolGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Default> DerefMut for PoolGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack() {
        let pool: PoolRef<u32> = Pool::build(PoolConfig {
            min_allocation: 150,
            max_allocation: 150,
        });
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
        let pool: PoolRef<u32> = Pool::build(PoolConfig {
            min_allocation: 150,
            max_allocation: 150,
        });
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
        let pool: PoolRef<u32> = Pool::build(PoolConfig {
            min_allocation: 150,
            max_allocation: 150,
        });

        let mut temp = pool.acquire().unwrap();
        *temp = 99;
        drop(temp);

        for _ in 0..1000 {
            let value = pool.acquire().unwrap();
            assert_eq!(*value, 99);
        }
    }

    #[test]
    fn test_stack_v4() {
        let pool: PoolRef<u32> = Pool::build(PoolConfig {
            min_allocation: 150,
            max_allocation: 200,
        });

        for _ in 0..200 {
            pool.acquire();
        }

        let mut guard_list = Vec::new();
        for _ in 0..200 {
            guard_list.push(pool.acquire());
        }

        assert!(pool.acquire().is_none());
    }

    #[test]
    fn test_stack_v5() {
        let pool: PoolRef<u32> = Pool::build(PoolConfig {
            min_allocation: 150,
            max_allocation: 170,
        });

        let mut guard_list = Vec::new();
        for _ in 0..150 {
            let value = pool.acquire();
            assert!(value.is_some());
            guard_list.push(value);
        }

        for _ in 0..20 {
            let value = pool.acquire();
            assert!(value.is_some());
            guard_list.push(value);
        }

        for _ in 0..50 {
            assert!(pool.acquire().is_none());
        }
    }

    #[test]
    fn test_stack_clone() {
        let pool: PoolRef<u32> = Pool::build(PoolConfig {
            min_allocation: 150,
            max_allocation: 150,
        });
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
