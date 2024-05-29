use anyhow::Result;

pub trait BinSearchExt {
    type Item;

    fn binsearch_key_map<B: Ord, T>(&self, b: &B, keyfn: impl FnMut(&Self::Item) -> B, mapfn: impl FnOnce(&Self::Item) -> Result<T>) -> Result<T>;
    fn binsearch_key_map_mut<B: Ord, T>(&mut self, b: &B, keyfn: impl FnMut(&Self::Item) -> B, mapfn: impl FnOnce(&mut Self::Item) -> Result<T>) -> Result<T>;
}

impl <I> BinSearchExt for [I] {
    type Item = I;

    #[inline]
    fn binsearch_key_map<B: Ord, T>(&self, b: &B, keyfn: impl FnMut(&Self::Item) -> B, mapfn: impl FnOnce(&Self::Item) -> Result<T>) -> Result<T> {
        self.binary_search_by_key(b, keyfn).map_or_else(
            |_| anyhow::bail!("not found"),
            |i| mapfn(&self[i]),
        )
    }

    #[inline]
    fn binsearch_key_map_mut<B: Ord, T>(&mut self, b: &B, keyfn: impl FnMut(&Self::Item) -> B, mapfn: impl FnOnce(&mut Self::Item) -> Result<T>) -> Result<T> {
        self.binary_search_by_key(b, keyfn).map_or_else(
            |_| anyhow::bail!("not found"),
            |i| mapfn(&mut self[i]),
        )
    }
}

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