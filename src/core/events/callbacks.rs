use std::marker::PhantomData;

/// Callback mode: `Callbacks<T>` defaults to by-ref callbacks (`FnMut(&T)`).
pub struct Ref;
/// Callback mode for mutable callbacks (`FnMut(&mut T)`).
pub struct Mut;
/// Callback mode for 2-arguments callbacks (`FnMut(&A, &B)`), used as `Callbacks<(A, B), Ref2>`.
pub struct Ref2;

pub trait CallbackMode<T> {
    type Callback: ?Sized;
}

impl<T> CallbackMode<T> for Ref {
    type Callback = dyn FnMut(&T);
}

impl<T> CallbackMode<T> for Mut {
    type Callback = dyn FnMut(&mut T);
}

impl<A, B> CallbackMode<(A, B)> for Ref2 {
    type Callback = dyn FnMut(&A, &B);
}

/// A single, centered callback list type.
///
/// - `Callbacks<T>` stores `FnMut(&T)` callbacks.
/// - `Callbacks<T, Mut>` stores `FnMut(&mut T)` callbacks.
/// - `Callbacks<(A, B), Ref2>` stores `FnMut(&A, &B)` callbacks.
pub struct Callbacks<T, Mode = Ref>
where
    Mode: CallbackMode<T>,
{
    callbacks: Vec<(usize, Box<Mode::Callback>)>,
    next_id: usize,
    _phantom: PhantomData<T>,
}

impl<T, Mode> Callbacks<T, Mode>
where
    Mode: CallbackMode<T>,
{
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
            next_id: 0,
            _phantom: PhantomData,
        }
    }

    pub fn remove(&mut self, id: usize) -> bool {
        let len_before = self.callbacks.len();
        self.callbacks.retain(|(cb_id, _)| *cb_id != id);
        self.callbacks.len() != len_before
    }
}

impl<T> Callbacks<T, Ref> {
    pub fn add<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&T) + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        self.callbacks.push((id, Box::new(f)));
        id
    }

    pub fn invoke(&mut self, arg: &T) {
        for (_, callback) in &mut self.callbacks {
            callback(arg);
        }
    }
}

impl<T, F> std::ops::AddAssign<F> for Callbacks<T, Ref>
where
    F: FnMut(&T) + 'static,
{
    fn add_assign(&mut self, rhs: F) {
        self.add(rhs);
    }
}

impl<T> Callbacks<T, Mut> {
    pub fn add<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&mut T) + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        self.callbacks.push((id, Box::new(f)));
        id
    }

    pub fn invoke(&mut self, arg: &mut T) {
        for (_, callback) in &mut self.callbacks {
            callback(arg);
        }
    }
}

impl<T, F> std::ops::AddAssign<F> for Callbacks<T, Mut>
where
    F: FnMut(&mut T) + 'static,
{
    fn add_assign(&mut self, rhs: F) {
        self.add(rhs);
    }
}

impl<A, B> Callbacks<(A, B), Ref2> {
    pub fn add<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&A, &B) + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        self.callbacks.push((id, Box::new(f)));
        id
    }

    pub fn invoke(&mut self, a: &A, b: &B) {
        for (_, callback) in &mut self.callbacks {
            callback(a, b);
        }
    }
}

impl<A, B, F> std::ops::AddAssign<F> for Callbacks<(A, B), Ref2>
where
    F: FnMut(&A, &B) + 'static,
{
    fn add_assign(&mut self, rhs: F) {
        self.add(rhs);
    }
}
