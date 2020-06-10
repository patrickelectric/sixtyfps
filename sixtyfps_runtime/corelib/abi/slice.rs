//! FFI-friendly slice

use core::{cmp::PartialEq, marker::PhantomData, ptr::NonNull};

/// That's basicaly the same as `&'a [T]`  but `repr(C)`
///
/// Can be constructed from a slice using the from trait.
///
/// ```
/// use sixtyfps_corelib::abi::slice::Slice;
/// let x = Slice::from_slice(&[1, 2, 3]);
/// assert_eq!(x.len(), 3);
/// assert_eq!(x[1], 2);
/// let slice : &'static [u32] = x.as_slice();
/// ```
///
/// Comparing two Slice compare their pointer, not the content.
/// ```
/// use sixtyfps_corelib::abi::slice::Slice;
/// let a = Slice::from_slice(&[1, 2, 3]);
/// let slice = [1, 2, 3, 4];
/// let b = Slice::from(&slice[..3]);
/// // two slice coming from the same pointer are equal.
/// assert_eq!(b, Slice::from(&slice[..3]));
/// // these are different because the pointers are different
/// assert_ne!(a, b);
/// // use as_slice to compare the contents
/// assert_eq!(a.as_slice(), b.as_slice());
/// ```
#[repr(C)]
#[derive(PartialEq, Debug)]
pub struct Slice<'a, T> {
    /// Invariant, this is a valid slice of len `len`
    ptr: NonNull<T>,
    len: usize,
    phantom: PhantomData<&'a [T]>,
}

// Need to implement manually otheriwse it is not implemented if T do not implement Copy / Clone
impl<'a, T> Copy for Slice<'a, T> {}

impl<'a, T> Clone for Slice<'a, T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr, len: self.len, phantom: PhantomData }
    }
}

impl<'a, T> Slice<'a, T> {
    /// Return a slice
    pub fn as_slice(self) -> &'a [T] {
        // Safety: it ptr is supposed to be a valid slice of given lenght
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Create from a native slice
    pub fn from_slice(x: &'a [T]) -> Self {
        x.into()
    }
}

impl<'a, T> From<&'a [T]> for Slice<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Slice { ptr: NonNull::from(slice).cast(), len: slice.len(), phantom: PhantomData }
    }
}

impl<'a, T> core::ops::Deref for Slice<'a, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}