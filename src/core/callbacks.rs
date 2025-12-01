type Callback<T> = Box<dyn FnMut(&T)>;

/// Small utility to register and invoke multiple callbacks.
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
