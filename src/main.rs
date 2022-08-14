use std::ptr;

pub trait ComputationalNode {
    fn evaluate(&mut self);
}

struct Output<T> {
    data: T,
}

impl<T: Default> Default for Output<T> {
    fn default() -> Output<T> {
        Output::<T> {
            data: Default::default(),
        }
    }
}

struct Input<T> {
    data: *const T,
}

impl<T> Input<T> {
    fn get(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T> Default for Input<T> {
    fn default() -> Input<T> {
        Input::<T> { data: ptr::null() }
    }
}

struct Node1 {
    output1: Output<f64>,
}

impl ComputationalNode for Node1 {
    fn evaluate(&mut self) {
        self.output1.data += 1.0
    }
}

struct Node2 {
    input1: Input<f64>,
}

impl ComputationalNode for Node2 {
    fn evaluate(&mut self) {
        println!("{}", self.input1.get())
    }
}

fn main() {
    let mut first = Node1 {
        output1: Default::default(),
    };
    let mut second = Node2 {
        input1: Default::default(),
    };
    second.input1.data = &first.output1.data;
    first.evaluate();
    first.evaluate();
    second.evaluate();
}
