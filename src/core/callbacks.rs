/// Type alias for a boxed callback function.
type Callback<T> = Box<dyn FnMut(&T)>;
/// Type alias for a boxed callback that needs mutable access.
type MutCallback<T> = Box<dyn FnMut(&mut T)>;

/// A struct that manages a list of callback functions.
pub struct Callbacks<T> {
    callbacks: Vec<(usize, Callback<T>)>,
    next_id: usize,
}

impl<T> Callbacks<T> {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&T) + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        self.callbacks.push((id, Box::new(f)));
        id
    }

    pub fn remove(&mut self, id: usize) -> bool {
        let len_before = self.callbacks.len();
        self.callbacks.retain(|(cb_id, _)| *cb_id != id);
        self.callbacks.len() != len_before
    }

    pub fn invoke(&mut self, arg: &T) {
        for (_, callback) in &mut self.callbacks {
            callback(arg);
        }
    }
}

impl<T, F> std::ops::AddAssign<F> for Callbacks<T>
where
    F: FnMut(&T) + 'static,
{
    fn add_assign(&mut self, rhs: F) {
        self.add(rhs);
    }
}

/// A struct that manages callbacks requiring mutable access to their argument.
pub struct CallbacksMut<T> {
    callbacks: Vec<(usize, MutCallback<T>)>,
    next_id: usize,
}

impl<T> CallbacksMut<T> {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&mut T) + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        self.callbacks.push((id, Box::new(f)));
        id
    }

    pub fn remove(&mut self, id: usize) -> bool {
        let len_before = self.callbacks.len();
        self.callbacks.retain(|(cb_id, _)| *cb_id != id);
        self.callbacks.len() != len_before
    }

    pub fn invoke(&mut self, arg: &mut T) {
        for (_, callback) in &mut self.callbacks {
            callback(arg);
        }
    }
}

impl<T, F> std::ops::AddAssign<F> for CallbacksMut<T>
where
    F: FnMut(&mut T) + 'static,
{
    fn add_assign(&mut self, rhs: F) {
        self.add(rhs);
    }
}
