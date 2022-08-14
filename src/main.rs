use std::ptr;
use std::ops::{Deref, DerefMut};


pub trait ComputationalNode {
    fn evaluate(& mut self);
}

struct Output<T> {
    data: T,
}

impl<T: Default> Default for Output<T> {
    fn default() -> Output<T> {
        Output::<T> { data: Default::default() }
    }
}


struct Input<T> {
    data: * const T,
}

impl<T> Input<T> {
    fn get(&self) -> &T {
        unsafe {
            &*self.data
        }
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

impl Node1 {
    fn new() -> Box<Self> {
        Box::new(Node1{output1: Default::default()})
    }
}


impl ComputationalNode for Node1 {
    fn evaluate(&mut self) {
        self.output1.data += 1.0
    }
}

struct Node2 {
    input1: Input<f64>,
}

impl Node2 {
    fn new() -> Box<Self> {
        Box::new(Node2{input1: Default::default()})
    }
}

impl ComputationalNode for Node2 {
    fn evaluate(&mut self) {
        println!("{}", self.input1.get())
    }
}


fn main() {
    let mut v = Vec::<Box<dyn ComputationalNode>>::new();
    let mut first = Node1::new();
    let mut second = Node2::new();
    second.input1.data = &first.output1.data;
    v.push(first);
    v.push(second);
    v[0].evaluate();
    v[1].evaluate();
    v[0].evaluate();
    v[1].evaluate();
}
