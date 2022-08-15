#![feature(strict_provenance)]

pub mod compute_graph {

    use core::cell::UnsafeCell;
    use std::any::Any;
    use std::collections::HashMap;
    use std::marker::PhantomData;
    use std::ptr;

    #[derive(Default)]
    pub struct Output<T> {
        data: UnsafeCell<T>,
    }

    impl<T> Output<T> {
        pub fn get(&self) -> &T {
            unsafe { &*self.data.get() }
        }
        pub fn get_mut(&mut self) -> &mut T {
            unsafe { &mut *self.data.get() }
        }
    }

    #[derive(Copy, Clone)]
    pub struct InputMaker {
        phantom: PhantomData<i32>,
    }

    pub struct Input<T> {
        data: UnsafeCell<*const UnsafeCell<T>>,
    }

    impl<T> Input<T> {
        pub fn get(&self) -> &T {
            unsafe { &*(**self.data.get()).get() }
        }
    }

    impl<T> Input<T> {
        pub fn new(_: InputMaker) -> Input<T> {
            Input::<T> {
                data: UnsafeCell::new(ptr::null()),
            }
        }
    }

    pub unsafe trait UnsafeNode {
        fn evaluate(&mut self);
        fn declare_attributes(&mut self, declare_info: Box<dyn Any>) -> NodeAttributes;
    }

    // These traits to be implemented automatically and safely by Derive macros
    pub unsafe trait OutputStruct {
        fn declare_outputs(&self) -> OutputAttributes;
    }

    pub unsafe trait InputStruct {
        fn new(_: InputMaker) -> Self;
    }

    // Safe trait, most users just implement this
    pub trait ComputationalNode {
        type Outputs: OutputStruct;
        type Inputs: InputStruct;
        type InitInfo;

        fn create_outputs(&mut self, init_info: &Self::InitInfo) -> Self::Outputs;

        fn initialize<'a>(
            &mut self,
            init_info: Self::InitInfo,
            inputs: &'a Self::Inputs,
        ) -> InputAttributes<'a>;

        fn evaluate(&mut self, inputs: &Self::Inputs, outputs: &mut Self::Outputs);
    }

    pub struct ErasedNode<T: ComputationalNode + 'static> {
        node: T,
        inputs: T::Inputs,
        outputs: T::Outputs,
    }

    unsafe impl<T: ComputationalNode + 'static> UnsafeNode for ErasedNode<T> {
        fn evaluate(&mut self) {
            self.node.evaluate(&self.inputs, &mut self.outputs);
        }
        fn declare_attributes(&mut self, declare_info: Box<dyn Any>) -> NodeAttributes {
            let mut output_attrs = self.outputs.declare_outputs();
            let input_attrs = self.node.initialize(
                *declare_info.downcast::<T::InitInfo>().unwrap(),
                &self.inputs,
            );
            NodeAttributes {
                inputs: input_attrs,
                outputs: output_attrs,
            }
        }
    }

    trait InputSetter {
        fn set(&mut self, target: &Box<dyn Any>);
    }

    impl<T: 'static> InputSetter for *const Input<T> {
        fn set(&mut self, target: &Box<dyn Any>) {
            match target.downcast_ref::<*const UnsafeCell<T>>() {
                None => {
                    assert!(false, "Input type and output type mismatch!")
                }
                Some(t) => unsafe {
                    *(**self).data.get() = *t;
                },
            }
        }
    }

    pub struct OutputAttributes<'a> {
        data: HashMap<String, Box<dyn Any>>,
        phantom: PhantomData<&'a dyn Any>,
    }

    impl<'a> OutputAttributes<'a> {
        pub fn new() -> OutputAttributes<'static> {
            OutputAttributes {
                data: HashMap::new(),
                phantom: PhantomData,
            }
        }

        pub fn add<T: 'static>(&mut self, name: String, output: &'a Output<T>) {
            self.data
                .insert(name, Box::new(&output.data as *const UnsafeCell<T>));
        }
    }

    pub struct InputAttributes<'a> {
        data: HashMap<String, Box<dyn InputSetter>>,
        phantom: PhantomData<&'a dyn Any>,
    }

    impl<'a> InputAttributes<'a> {
        pub fn new() -> InputAttributes<'static> {
            InputAttributes {
                data: HashMap::new(),
                phantom: PhantomData,
            }
        }
        pub fn add<T: 'static>(&mut self, name: String, input: &'a Input<T>) {
            self.data.insert(name, Box::new(input as *const Input<T>));
        }
    }

    pub struct NodeAttributes<'a> {
        pub inputs: InputAttributes<'a>,
        pub outputs: OutputAttributes<'a>,
    }

    pub struct DeclaredNode {
        node: Box<dyn UnsafeNode + 'static>,
        init_info: Box<dyn Any>,
    }

    impl DeclaredNode {
        pub fn new<T: ComputationalNode + 'static>(
            mut node: T,
            mut init_info: T::InitInfo,
        ) -> DeclaredNode {
            let inputs = T::Inputs::new(InputMaker {
                phantom: PhantomData,
            });
            let outputs = node.create_outputs(&init_info);
            let node = Box::new(ErasedNode {
                node,
                inputs,
                outputs,
            });
            let init_info = Box::new(init_info);
            DeclaredNode { node, init_info }
        }
    }

    pub struct GraphBuilder {
        nodes: Vec<Box<dyn UnsafeNode>>,
        inputs: HashMap<String, Vec<Box<dyn InputSetter>>>,
        outputs: HashMap<String, Box<dyn Any>>,
    }

    pub struct Graph {
        nodes: Vec<Box<dyn UnsafeNode>>,
    }

    impl GraphBuilder {
        pub fn new() -> GraphBuilder {
            GraphBuilder {
                nodes: Vec::new(),
                inputs: HashMap::new(),
                outputs: HashMap::new(),
            }
        }
        pub fn add(&mut self, name: &str, declared_node: DeclaredNode) {
            let mut node = declared_node.node;
            let attributes = node.declare_attributes(declared_node.init_info);

            for (key, value) in attributes.inputs.data {
                if !self.inputs.contains_key(&key) {
                    self.inputs.insert(key.clone(), Vec::new());
                }
                self.inputs.get_mut(&key).unwrap().push(value);
            }
            for (key, value) in attributes.outputs.data {
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
            Graph { nodes: self.nodes }
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