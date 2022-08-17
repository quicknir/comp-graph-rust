#![feature(strict_provenance)]

pub mod compute_graph {
    use std::any::Any;
    use std::collections::HashMap;
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use std::ptr;

    struct AliasBox<T: ?Sized> {
        ptr: *mut T,
        phantom: PhantomData<T>,
    }

    impl<T: ?Sized> AliasBox<T> {
        fn new(t: Box<T>) -> AliasBox<T> {
            AliasBox {
                ptr: Box::into_raw(t),
                phantom: PhantomData,
            }
        }
    }

    impl<T: ?Sized> Drop for AliasBox<T> {
        fn drop(&mut self) {
            unsafe { Box::from_raw(self.ptr) };
        }
    }

    #[derive(Default)]
    pub struct Output<T> {
        data: T,
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

    #[derive(Copy, Clone)]
    pub struct InputMaker {
        phantom: PhantomData<i32>,
    }

    pub struct Input<T> {
        data: *const T,
    }

    impl<T> Deref for Input<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { &(*self.data) }
        }
    }

    impl<T> Input<T> {
        pub fn new(_: InputMaker) -> Input<T> {
            Input::<T> { data: ptr::null() }
        }
    }
    pub unsafe trait UnsafeNode {
        fn evaluate(&mut self);
        fn declare_attributes<'a>(
            &'a mut self,
            declare_info: Box<dyn Any>,
            node_attributes: &mut NodeAttributes<'a>,
        );
    }

    // These traits to be implemented automatically and safely by Derive macros
    pub unsafe trait OutputStruct {
        fn declare_outputs<'a>(&'a self, outputs: &mut OutputAttributes<'a>);
    }

    pub unsafe trait InputStruct {
        fn new(_: InputMaker) -> Self;
        fn declare_inputs<'a>(&'a mut self, inputs: &mut InputAttributes<'a>);
    }

    // Safe trait, most users just implement this
    pub trait ComputationalNode {
        type Outputs: OutputStruct;
        type Inputs: InputStruct;
        type InitInfo;

        fn initialize(
            &mut self,
            init_info: Self::InitInfo,
            input_atts: BoundInputs,
            outputs_atts: BoundOutputs,
        );

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
        fn declare_attributes<'a>(
            &'a mut self,
            declare_info: Box<dyn Any>,
            node_attributes: &mut NodeAttributes<'a>,
        ) {
            self.inputs.declare_inputs(&mut node_attributes.inputs);
            self.outputs.declare_outputs(&mut node_attributes.outputs);
            self.node.initialize(
                *declare_info.downcast::<T::InitInfo>().unwrap(),
                node_attributes.inputs.bind(),
                node_attributes.outputs.bind(),
            );
        }
    }

    trait InputSetter {
        fn set(&mut self, target: &Box<dyn Any>) -> Result<(), ()>;
    }

    impl<T: 'static> InputSetter for *mut Input<T> {
        fn set(&mut self, target: &Box<dyn Any>) -> Result<(), ()> {
            let t = target.downcast_ref::<*const T>().ok_or(())?;
            unsafe { (**self).data = *t };
            Ok(())
        }
    }

    pub struct OutputAttributes<'a> {
        data: HashMap<String, Box<dyn Any>>,
        phantom: PhantomData<&'a dyn Any>,
        master_ptr: *mut u8,
    }

    impl<'a> OutputAttributes<'a> {
        pub fn new(master_ptr: *mut dyn UnsafeNode) -> OutputAttributes<'static> {
            OutputAttributes {
                data: HashMap::new(),
                phantom: PhantomData,
                master_ptr: master_ptr as *mut u8,
            }
        }

        pub fn add<T: 'static>(&mut self, name: &str, output: &'a Output<T>) {
            let mut input = &output.data as *const T;
            input = self.master_ptr.with_addr(input.addr()).cast();
            self.data.insert(name.to_string(), Box::new(input));
        }

        pub fn bind<'b>(&'b mut self) -> BoundOutputs<'a, 'b> {
            BoundOutputs { atts: self }
        }
    }

    pub struct BoundOutputs<'a, 'b> {
        atts: &'b mut OutputAttributes<'a>,
    }

    impl<'a, 'b> BoundOutputs<'a, 'b> {
        pub fn rename(&mut self, old_name: &str, new_name: &str) {
            let (_, input) = self.atts.data.remove_entry(old_name).unwrap();
            self.atts.data.insert(new_name.to_string(), input);
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
        pub fn add<T: 'static>(&mut self, name: &str, input: &'a mut Input<T>) {
            self.data.insert(name.to_string(), Box::new(input as *mut Input<T>));
        }
        pub fn bind<'b>(&'b mut self) -> BoundInputs<'a, 'b> {
            BoundInputs { atts: self }
        }
    }

    pub struct BoundInputs<'a, 'b> {
        atts: &'b mut InputAttributes<'a>,
    }

    impl<'a, 'b> BoundInputs<'a, 'b> {
        pub fn rename(&mut self, old_name: &str, new_name: &str) {
            let (_, input) = self.atts.data.remove_entry(old_name).unwrap();
            self.atts.data.insert(new_name.to_string(), input);
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
            node: T,
            init_info: T::InitInfo,
            outputs: T::Outputs,
        ) -> DeclaredNode {
            let inputs = T::Inputs::new(InputMaker {
                phantom: PhantomData,
            });
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
        nodes: Vec<AliasBox<dyn UnsafeNode>>,
        inputs: HashMap<String, Vec<Box<dyn InputSetter>>>,
        outputs: HashMap<String, Box<dyn Any>>,
    }

    pub struct Graph {
        nodes: Vec<AliasBox<dyn UnsafeNode>>,
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
            let node = AliasBox::new(declared_node.node);
            let mut attributes = NodeAttributes {
                inputs: InputAttributes::new(),
                outputs: OutputAttributes::new(node.ptr),
            };
            unsafe { (*node.ptr).declare_attributes(declared_node.init_info, &mut attributes) };
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
                    let output_lookup = self.outputs.get(key);
                    assert!(
                        output_lookup.is_some(),
                        "Error, no output for input {}",
                        key
                    );
                    let result = input_setter.set(output_lookup.unwrap());
                    assert!(
                        result.is_ok(),
                        "Error, input and output types mismatch at {}",
                        key
                    );
                }
            }
            Graph { nodes: self.nodes }
        }
    }

    impl Graph {
        pub fn evaluate(&mut self) {
            for n in &mut self.nodes {
                unsafe {
                    (*n.ptr).evaluate();
                }
            }
        }
    }
}
