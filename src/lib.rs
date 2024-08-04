#![no_std]
#[warn(missing_docs)]
extern crate alloc;

/// Trait for consuming types by value.
pub trait Finalize {
    fn finalize(self);
}

impl<T: FnOnce()> Finalize for T {
    fn finalize(self) {
        self()
    }
}

pub use crate::auto_finalizer::*;
pub use crate::finalizer::*;

mod finalizer {
    use super::Finalize;
    use core::mem::ManuallyDrop;
    use core::ops::{Deref, DerefMut};

    #[repr(transparent)]
    #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AutoFinalizer<T: Finalize> {
        inner: ManuallyDrop<T>,
    }

    impl<T: Finalize> AutoFinalizer<T> {
        #[inline]
        pub const fn new(value: T) -> Self {
            Self {
                inner: ManuallyDrop::new(value),
            }
        }

        #[inline]
        pub fn into_inner(item: Self) -> T {
            unsafe {
                let mut item = ManuallyDrop::new(item);
                ManuallyDrop::take(&mut item.inner)
            }
        }
    }

    impl<T: Finalize> Deref for AutoFinalizer<T> {
        type Target = T;
        #[inline]
        fn deref(&self) -> &Self::Target {
            self.inner.deref()
        }
    }

    impl<T: Finalize> DerefMut for AutoFinalizer<T> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.inner.deref_mut()
        }
    }

    impl<T: Finalize> Drop for AutoFinalizer<T> {
        #[inline]
        fn drop(&mut self) {
            unsafe { ManuallyDrop::take(&mut self.inner).finalize() }
        }
    }
}

mod auto_finalizer {
    use super::AutoFinalizer;
    use crate::Finalize;
    use core::ops::{Deref, DerefMut};

    pub trait Terminator<T> {
        fn terminate(self, other: T);
    }

    impl<T, F: FnOnce(T)> Terminator<T> for F {
        #[inline]
        fn terminate(self, other: T) {
            self(other)
        }
    }

    #[derive(Default, Debug, Clone)]
    struct TermPair<T, F>(T, F);

    impl<T, F: Terminator<T>> Finalize for TermPair<T, F> {
        #[inline]
        fn finalize(self) {
            self.1.terminate(self.0)
        }
    }

    #[derive(Default, Debug, Clone)]
    pub struct ScopedTerminator<T, F: Terminator<T>> {
        inner: AutoFinalizer<TermPair<T, F>>,
    }

    impl<T, F: Terminator<T>> ScopedTerminator<T, F> {
        #[inline]
        pub const fn new(value: T, terminator: F) -> Self {
            Self {
                inner: AutoFinalizer::new(TermPair(value, terminator)),
            }
        }

        #[inline]
        pub fn into_pair(item: Self) -> (T, F) {
            let pair = AutoFinalizer::into_inner(item.inner);
            (pair.0, pair.1)
        }
    }

    impl<T, F: Terminator<T>> Deref for ScopedTerminator<T, F> {
        type Target = T;
        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.inner.deref().0
        }
    }

    impl<T, F: Terminator<T>> DerefMut for ScopedTerminator<T, F> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner.deref_mut().0
        }
    }
}
