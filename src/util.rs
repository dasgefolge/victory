use {
    enum_iterator::IntoEnumIterator,
    itertools::Itertools as _,
};

pub(crate) trait Cycle {
    /// Returns the element of the cycle that comes before `self`.
    ///
    /// # Panics
    ///
    /// This method panics if the cycle is empty.
    fn prev(&self) -> Self;
    /// Returns the element of the cycle that comes after `self`.
    ///
    /// # Panics
    ///
    /// This method panics if the cycle is empty.
    fn succ(&self) -> Self;
}

impl<T: IntoEnumIterator + Eq> Cycle for T {
    fn prev(&self) -> T {
        if T::into_enum_iter().next().map_or(true, |next| next == *self) {
            T::into_enum_iter().last().expect("empty cycle")
        } else {
            let mut prev = None;
            for elt in T::into_enum_iter() {
                if elt == *self { return prev.expect("self is first") }
                prev = Some(elt);
            }
            panic!("self not found")
        }
    }

    fn succ(&self) -> T {
        if T::into_enum_iter().last().map_or(true, |last| last == *self) {
            T::into_enum_iter().next().expect("empty cycle")
        } else {
            let mut iter = T::into_enum_iter();
            let _ = iter.find(|elt| elt == self).expect("self not found");
            iter.next().expect("self was last")
        }
    }
}
