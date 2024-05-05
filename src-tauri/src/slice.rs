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