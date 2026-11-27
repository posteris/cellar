use crate::diagnostics::watch::Watch;

pub struct Monitor<const N: usize> {
    watches: [Watch; N],
}

impl<const N: usize> Monitor<N> {
    #[inline]
    pub fn new() -> Self {
        let watches = [Watch::new(); N];
        Self { watches }
    }

    #[inline]
    pub fn watch(&mut self, index: usize) -> &mut Watch {
        &mut self.watches[index]
    }
}
