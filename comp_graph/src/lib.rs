#![feature(strict_provenance)]

pub mod compute_graph {

use std::collections::HashMap;
use std::ptr;
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
        assert!(self.data != ptr::null());
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

pub struct NodeAttributes {
    inputs: HashMap<String, * mut Input<f64>>,
    outputs: HashMap<String, * const f64>,
}

impl NodeAttributes {
    pub fn new() -> NodeAttributes {
        NodeAttributes{inputs: HashMap::new(), outputs: HashMap::new() }
    }

    pub fn add_input(&mut self, name: String, input: &mut Input<f64>) {
        self.inputs.insert(name, input as * mut Input<f64>);
    }

    pub fn add_output(&mut self, name: String, output: & Output<f64>) {
        self.outputs.insert(name, &output.data as * const f64);
    }
}

pub struct DeclaredNode {
    pub node: Box<dyn ComputationalNode>,
    pub attributes: NodeAttributes,
}


pub struct GraphBuilder {
    nodes: Vec::<Box<dyn ComputationalNode>>,
    inputs: HashMap<String, * mut Input<f64>>,
    outputs: HashMap<String, * const f64>,
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
        for (key, value) in & declared_node.attributes.inputs {
            self.inputs.insert(key.clone(), *value);
        }
        for (key, value) in & declared_node.attributes.outputs {
            self.outputs.insert(format!("{name}.{key}"), *value);
        }

   }
   pub fn build(self) -> Graph {
       for (key, value) in self.inputs {
           unsafe {
               (*value).data = *self.outputs.get(&key).unwrap();
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
