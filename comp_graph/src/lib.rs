#![feature(strict_provenance)]

pub mod compute_graph {

use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::any::Any;
use std::ops::{Deref, DerefMut};


pub trait ComputationalNode {
    fn evaluate(&mut self);
}

pub struct Output<T> {
    data: T,
}

impl<T: Default> Default for Output<T> {
    fn default() -> Output<T> {
        Output::<T> { data: Default::default() }
    }
}

impl<T> Deref for Output<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Output<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub struct Input<T> {
    data: * const T,
}

impl<T> Deref for Input<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        assert!(self.data != ptr::null(), "This should be impossible");
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

trait InputSetter {
    fn set(&mut self, target: & Box<dyn Any>);
}

impl<T: 'static> InputSetter for * mut Input<T> {
    fn set(&mut self, target: & Box<dyn Any>) {
        match target.downcast_ref::<* const T>() {
            None => { assert!(false, "Input type and output type mismatc") }
            Some(t) => {
                unsafe {
                    (**self).data = *t;
                }
            }
        }
    }
}

pub struct NodeAttributes {
    inputs: HashMap<String, Box<dyn InputSetter>>,
    outputs: HashMap<String, Box<dyn Any>>,
}

impl NodeAttributes {
    pub fn new() -> NodeAttributes {
        NodeAttributes{inputs: HashMap::new(), outputs: HashMap::new() }
    }

    pub fn add_input<T: 'static>(&mut self, name: String, input: &mut Input<T>) {
        self.inputs.insert(name, Box::new(input as * mut Input<T>));
    }

    pub fn add_output<T: 'static>(&mut self, name: String, output: & Output<T>) {
        self.outputs.insert(name, Box::new(&output.data as * const T));
    }
}

pub struct DeclaredNode {
    pub node: Box<dyn ComputationalNode>,
    pub attributes: NodeAttributes,
}


pub struct GraphBuilder {
    nodes: Vec::<Box<dyn ComputationalNode>>,
    inputs: HashMap<String, Vec::<Box<dyn InputSetter>>>,
    outputs: HashMap<String, Box<dyn Any>>,
}

pub struct Graph {
    nodes: Vec::<Box<dyn ComputationalNode>>,    
}


impl GraphBuilder {
   pub fn new() -> GraphBuilder {
       GraphBuilder{nodes: Vec::new(), inputs: HashMap::new(), outputs: HashMap::new() }
   }
   pub fn add(&mut self, name: String,  declared_node: DeclaredNode) {
        self.nodes.push(declared_node.node);
        for (key, value) in declared_node.attributes.inputs {
            if !self.inputs.contains_key(&key) {
                self.inputs.insert(key.clone(), Vec::new());
            }
            self.inputs.get_mut(&key).unwrap().push(value);
        }
        for (key, value) in declared_node.attributes.outputs {
            self.outputs.insert(format!("{name}.{key}"), value);
        }

   }
   pub fn build(mut self) -> Graph {
       for (key, value) in &mut self.inputs {
           for input_setter in value {
               input_setter.set(self.outputs.get(key).unwrap());
           }
       }
       Graph{nodes: self.nodes}
   }
}

impl Graph {
    pub fn evaluate(&mut self) {
        for n in &mut self.nodes {
             n.evaluate();
        }
    }
}

}