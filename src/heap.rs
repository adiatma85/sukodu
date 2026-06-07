pub struct Heap {
    size: usize,
    data: Vec<i32>,
    id: Vec<usize>,
    index: Vec<usize>,
}

impl Heap {
    /// Creates a new empty Heap with room for up to `capacity` unique IDs.
    pub fn new(capacity: usize) -> Self {
        Heap {
            size: 0,
            data: Vec::with_capacity(capacity),
            id: Vec::with_capacity(capacity),
            index: vec![0; capacity],
        }
    }

    /// Swaps the elements at heap positions `i` and `j` and updates the index mapping.
    fn swap(&mut self, i: usize, j: usize) {
        self.data.swap(i, j);
        self.id.swap(i, j);
        self.index[self.id[i]] = i;
        self.index[self.id[j]] = j;
    }

    /// Sifts up the element at heap position `i`.
    fn sift_up(&mut self, mut i: usize) {
        while i > 0 {
            let j = (i - 1) / 2;
            if self.data[i] < self.data[j] {
                self.swap(i, j);
                i = j;
            } else {
                break;
            }
        }
    }

    /// Sifts down the element at heap position `i`.
    fn sift_down(&mut self, mut i: usize) {
        loop {
            let mut j = 2 * i + 1;
            if j >= self.size {
                break;
            }
            if j + 1 < self.size && self.data[j + 1] < self.data[j] {
                j += 1;
            }
            if self.data[j] < self.data[i] {
                self.swap(i, j);
                i = j;
            } else {
                break;
            }
        }
    }

    /// Appends ID `u` with priority value `x` to the end of the heap.
    pub fn append(&mut self, u: usize, x: i32) {
        let i = self.data.len();
        self.data.push(x);
        self.id.push(u);
        self.index[u] = i;
        if i > self.size {
            self.swap(i, self.size);
        }
        self.size += 1;
    }

    /// Re-establishes the heap property from the bottom up.
    pub fn heapify(&mut self) {
        if self.size == 0 {
            return;
        }
        let mut i = (self.size - 1) / 2;
        loop {
            self.sift_down(i);
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    /// Returns whether the active heap is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Removes and returns the ID with the minimum priority value.
    pub fn take_min(&mut self) -> usize {
        self.size -= 1;
        self.swap(0, self.size);
        self.sift_down(0);
        self.id[self.size]
    }

    /// Increases the priority value of ID `u` to `x`, sifting it down.
    pub fn increase(&mut self, u: usize, x: i32) {
        let i = self.index[u];
        if i < self.size {
            self.data[i] = x;
            self.sift_down(i);
        }
    }

    /// Decreases the priority value of ID `u` to `x`, sifting it up.
    pub fn decrease(&mut self, u: usize, x: i32) {
        let i = self.index[u];
        if i < self.size {
            self.data[i] = x;
            self.sift_up(i);
        }
    }

    /// Temporarily removes ID `u` from the active heap.
    pub fn remove(&mut self, u: usize) {
        let i = self.index[u];
        if i < self.size {
            self.size -= 1;
            self.swap(i, self.size);
            self.sift_down(i);
        }
    }

    /// Restores ID `u` back to the active heap.
    pub fn restore(&mut self, u: usize) {
        let i = self.index[u];
        if i >= self.size {
            self.swap(i, self.size);
            self.sift_up(self.size);
            self.size += 1;
        }
    }
}
