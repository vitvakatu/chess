//! Like std::iter::TakeWhile, but returns the first non-matching element as well.

pub struct TakeWhileInclusive<I, P> {
    iter: I,
    flag: bool,
    predicate: P,
}

impl<I, P> TakeWhileInclusive<I, P> {
    fn new(iter: I, predicate: P) -> Self {
        Self {
            iter,
            flag: false,
            predicate,
        }
    }
}

impl<I: Iterator, P> Iterator for TakeWhileInclusive<I, P>
where
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.flag {
            None
        } else {
            let x = self.iter.next()?;
            if !(self.predicate)(&x) {
                self.flag = true;
            }
            Some(x)
        }
    }
}

pub trait TakeWhileInclusiveExt: Iterator + Sized {
    fn take_while_inclusive<P: FnMut(&Self::Item) -> bool>(
        self,
        p: P,
    ) -> TakeWhileInclusive<Self, P>;
}

impl<T: Iterator> TakeWhileInclusiveExt for T {
    fn take_while_inclusive<P>(self, p: P) -> TakeWhileInclusive<Self, P> {
        TakeWhileInclusive::new(self, p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let x = vec![2i32, 4, 6, 9, 10];
        let y: Vec<_> = x
            .into_iter()
            .take_while_inclusive(|x: &i32| *x % 2 == 0)
            .collect();
        assert_eq!(y, vec![2, 4, 6, 9]);
    }
}
