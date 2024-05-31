
pub trait ExtendSelf: Default + Sized {
    fn extend(&mut self, other: &Self);
    fn folding<R: AsRef<Self>>(mut self, other: R) -> Self {
        self.extend(other.as_ref());
        self
    }
    #[inline]
    fn fold_iter<R: AsRef<Self>, I: IntoIterator<Item = R>>(iter: I) -> Self {
        iter.into_iter().fold(Self::default(), Self::folding)
    }
}

#[macro_export]
macro_rules! iter_extend {
    ($t:ty) => {
        impl <R> FromIterator<R> for $t where R: AsRef<Self> {
            #[inline]
            fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
                Self::fold_iter(iter)
            }
        }
    };
}
pub use iter_extend;