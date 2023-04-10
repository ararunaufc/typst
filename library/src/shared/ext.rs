//! Extension traits.

use std::marker::PhantomData;

use crate::layout::{AlignElem, MoveElem, PadElem};
use crate::prelude::*;
use crate::text::{EmphElem, FontFamily, FontList, StrongElem, TextElem, UnderlineElem};

/// Additional methods on content.
pub trait ContentExt {
    /// Make this content strong.
    fn strong(self) -> Self;

    /// Make this content emphasized.
    fn emph(self) -> Self;

    /// Underline this content.
    fn underlined(self) -> Self;

    /// Link the content somewhere.
    fn linked(self, dest: Destination) -> Self;

    /// Set alignments for this content.
    fn aligned(self, aligns: Axes<Option<GenAlign>>) -> Self;

    /// Pad this content at the sides.
    fn padded(self, padding: Sides<Rel<Length>>) -> Self;

    /// Transform this content's contents without affecting layout.
    fn moved(self, delta: Axes<Rel<Length>>) -> Self;
}

impl ContentExt for Content {
    fn strong(self) -> Self {
        StrongElem::new(self).pack()
    }

    fn emph(self) -> Self {
        EmphElem::new(self).pack()
    }

    fn underlined(self) -> Self {
        UnderlineElem::new(self).pack()
    }

    fn linked(self, dest: Destination) -> Self {
        self.styled(MetaElem::set_data(vec![Meta::Link(dest)]))
    }

    fn aligned(self, aligns: Axes<Option<GenAlign>>) -> Self {
        self.styled(AlignElem::set_alignment(aligns))
    }

    fn padded(self, padding: Sides<Rel<Length>>) -> Self {
        PadElem::new(self)
            .with_left(padding.left)
            .with_top(padding.top)
            .with_right(padding.right)
            .with_bottom(padding.bottom)
            .pack()
    }

    fn moved(self, delta: Axes<Rel<Length>>) -> Self {
        MoveElem::new(self).with_dx(delta.x).with_dy(delta.y).pack()
    }
}

/// Additional methods for style lists.
pub trait StylesExt {
    /// Set a font family composed of a preferred family and existing families
    /// from a style chain.
    fn set_family(&mut self, preferred: FontFamily, existing: StyleChain);
}

impl StylesExt for Styles {
    fn set_family(&mut self, preferred: FontFamily, existing: StyleChain) {
        self.set(TextElem::set_font(FontList(
            std::iter::once(preferred)
                .chain(TextElem::font_in(existing))
                .collect(),
        )));
    }
}
/// [`Iterator`] extension trait for fail-fast operations.
pub trait FailFastIteratorExt: Iterator {
    /// Maps [`Iterator`] values with a custom function when the [`Iterator::Item`]s
    /// are [`Result`]s.
    ///
    /// Upon the first time that the original `Iterator` yields a [`Result::Err`],
    /// the `Iterator` resulting from this function will yield that value and
    /// then refuse to yield another value.
    fn try_map_ok<T, E, F, U>(self, mapfn: F) -> FailFastMapIter<Self, T, E, F>
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
        F: FnMut(T) -> U,
    {
        FailFastMapIter {
            iter: self,
            bail: false,
            mapfn,
            _phantom: PhantomData,
        }
    }

    /// Folds [`Iterator`]s values with a custom function when the [`Iterator::Item`]s
    /// are [`Result`]s.
    ///
    /// This function operates only on the [`Result::Ok`] values yielded by the
    /// iterator. Upon the first time that the original `Iterator` yields a
    /// [`Result::Err`], this function will return that value and stop processing
    /// the remaining items.
    fn try_fold_ok<T, E, B, F>(mut self, initial: B, mut foldfn: F) -> Result<B, E>
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
        F: FnMut(B, T) -> B,
    {
        let mut folded = initial;

        loop {
            match self.next() {
                Some(Ok(value)) => {
                    folded = foldfn(folded, value);
                }
                Some(Err(error)) => return Err(error),
                None => return Ok(folded),
            }
        }
    }
}

impl<I, T, E> FailFastIteratorExt for I where I: Iterator<Item = Result<T, E>> {}

/// An [`Iterator`] that will short-circuit some of its operations.
///
/// This Iterator's accepts [`Result`]s as its [`Item`][`Iterator::Item`]s and,
/// when that yields a [`Result::Err`] value for the first time, it is guaranteed
/// that this will never yield another item.
pub struct FailFastMapIter<I, T, E, F> {
    bail: bool,
    iter: I,
    mapfn: F,
    // ? QUESTION: Is this correct from variance perspective?
    _phantom: PhantomData<(T, E)>,
}

impl<I, T, E, F, U> Iterator for FailFastMapIter<I, T, E, F>
where
    I: Iterator<Item = Result<T, E>>,
    F: FnMut(T) -> U,
{
    type Item = Result<U, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bail {
            None
        } else {
            match self.iter.next() {
                Some(Ok(value)) => Some(Ok((self.mapfn)(value))),
                Some(Err(error)) => {
                    self.bail = true;
                    Some(Err(error))
                }
                None => {
                    self.bail = true;
                    None
                }
            }
        }
    }
}
