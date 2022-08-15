#![feature(strict_provenance)]

pub mod compute_graph {

use std::collections::HashMap;
use std::any::Any;
use std::marker::PhantomData;
use core::cell::UnsafeCell;
use std::ptr;


pub trait ComputationalNode {
    type Attributes;
    type DeclareInfo;
    fn evaluate(&mut self, atts: &Self::Attributes);
    fn declare_attributes(atts: &Self::Attributes, declare_info: Self::DeclareInfo) -> NodeAttributes;
}

trait ErasedNode {
    fn evaluate(&mut self);
    fn declare_attributes(&self, declare_info: Box<dyn Any>) -> NodeAttributes;
}

struct ErasedNodeImpl<T: ComputationalNode + 'static> {
    node: T,
    atts: T::Attributes,
}

impl<T: ComputationalNode + 'static> ErasedNode for ErasedNodeImpl<T> {
    fn evaluate(&mut self) { self.node.evaluate(&self.atts); }
    fn declare_attributes(&self, declare_info: Box<dyn Any>) -> NodeAttributes  { 
        T::declare_attributes(& self.atts, *declare_info.downcast::<T::DeclareInfo>().unwrap())
    }
}

pub struct Output<T> {
    data: UnsafeCell<T>,
}

impl<T: Default> Default for Output<T> {
    fn default() -> Output<T> {
        Output::<T> { data: Default::default() }
    }
}

impl<T> Output<T> {
    pub fn get(&self) -> &T { unsafe { &*self.data.get() } } 
    pub fn get_mut(&self) -> &mut T { unsafe { &mut *self.data.get() } } 
}


pub struct Input<T> {
    data: UnsafeCell<* const UnsafeCell<T>>,
}

impl <T> Input<T> {
    pub fn get(&self) -> &T { unsafe { &*(**self.data.get()).get() } } 
}

impl<T> Default for Input<T> {
    fn default() -> Input<T> {
        Input::<T> { data: UnsafeCell::new(ptr::null()) }
    }
}

trait InputSetter {
    fn set(&mut self, target: & Box<dyn Any>);
}

impl<T: 'static> InputSetter for * const Input<T> {
    fn set(&mut self, target: & Box<dyn Any>) {
        match target.downcast_ref::<* const UnsafeCell<T>>() {
            None => { assert!(false, "Input type and output type mismatch!") }
            Some(t) => {
                unsafe {
                    *(**self).data.get() = *t;
                }
            }
        }
    }
}

pub struct NodeAttributes<'a> {
    inputs: HashMap<String, Box<dyn InputSetter>>,
    outputs: HashMap<String, Box<dyn Any>>,
    phantom: PhantomData<&'a dyn Any>,
}

impl <'a> NodeAttributes<'a> {
    pub fn new() -> NodeAttributes<'static> {
        NodeAttributes{inputs: HashMap::new(), outputs: HashMap::new(), phantom: PhantomData }
    }

    pub fn add_input<T: 'static>(&mut self, name: String, input: &'a Input<T>) {
        self.inputs.insert(name, Box::new(input as * const Input<T>));
    }

    pub fn add_output<T: 'static>(&mut self, name: String, output: &'a Output<T>) {
        self.outputs.insert(name, Box::new(&output.data as * const UnsafeCell<T>));
    }
}


pub struct DeclaredNode {
    node: Box<dyn ErasedNode + 'static>,
    declared_info: Box<dyn Any>,
}

impl DeclaredNode {
    pub fn new<T: ComputationalNode + 'static>(node: T, atts: T::Attributes, static_declared_info: T::DeclareInfo) -> DeclaredNode {
        let node = Box::new(ErasedNodeImpl{node, atts});
        let declared_info = Box::new(static_declared_info);
        DeclaredNode{node, declared_info}
    }
}


pub struct GraphBuilder {
    nodes: Vec<Box<dyn ErasedNode>>,
    inputs: HashMap<String, Vec::<Box<dyn InputSetter>>>,
    outputs: HashMap<String, Box<dyn Any>>,
}

pub struct Graph {
    nodes: Vec<Box<dyn ErasedNode>>,    
}


impl GraphBuilder {
   pub fn new() -> GraphBuilder {
       GraphBuilder{nodes: Vec::new(), inputs: HashMap::new(), outputs: HashMap::new() }
   }
   pub fn add(&mut self, name: &str, declared_node: DeclaredNode) {
        let mut node = declared_node.node;
        let attributes = node.declare_attributes(declared_node.declared_info);

        for (key, value) in attributes.inputs {
            if !self.inputs.contains_key(&key) {
                self.inputs.insert(key.clone(), Vec::new());
            }
            self.inputs.get_mut(&key).unwrap().push(value);
        }
        for (key, value) in attributes.outputs {
            self.outputs.insert(format!("{name}.{key}"), value);
        }
        
        self.nodes.push(node);
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





