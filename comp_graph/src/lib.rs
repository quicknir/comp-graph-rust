#![feature(strict_provenance)]

pub mod compute_graph {
    use core::cell::UnsafeCell;
    use std::any::Any;
    use std::collections::HashMap;
    use std::marker::PhantomData;
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
        fn declare_attributes(
            &mut self,
            master_ptr: *mut dyn UnsafeNode,
            declare_info: Box<dyn Any>,
        ) -> NodeAttributes;
    }

    // These traits to be implemented automatically and safely by Derive macros
    pub unsafe trait OutputStruct {
        fn declare_outputs(&self, master_ptr: *mut dyn UnsafeNode) -> BoundOutputs;
    }

    pub unsafe trait InputStruct {
        fn new(_: InputMaker) -> Self;
        fn declare_inputs(&self) -> BoundInputs;
    }

    // Safe trait, most users just implement this
    pub trait ComputationalNode {
        type Outputs: OutputStruct;
        type Inputs: InputStruct;
        type InitInfo;

        fn initialize(
            &mut self,
            init_info: Self::InitInfo,
            input_atts: &mut BoundInputs,
            outputs_atts: &mut BoundOutputs,
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
        fn declare_attributes(
            &mut self,
            master_ptr: *mut dyn UnsafeNode,
            declare_info: Box<dyn Any>,
        ) -> NodeAttributes {
            let mut bound_outputs = self.outputs.declare_outputs(master_ptr);
            let mut bound_inputs = self.inputs.declare_inputs();
            self.node.initialize(
                *declare_info.downcast::<T::InitInfo>().unwrap(),
                &mut bound_inputs,
                &mut bound_outputs,
            );
            NodeAttributes {
                inputs: bound_inputs.atts,
                outputs: bound_outputs.atts,
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

        pub fn add<T: 'static>(
            &mut self,
            name: String,
            output: &'a Output<T>,
            master_ptr: *mut dyn UnsafeNode,
        ) {
            let mut input = &output.data as *const UnsafeCell<T>;
            input = (master_ptr as *mut u8).with_addr(input.addr()).cast();
            self.data.insert(name, Box::new(input));
        }

        pub fn bind(self) -> BoundOutputs<'a> {
            BoundOutputs { atts: self }
        }
    }

    pub struct BoundOutputs<'a> {
        atts: OutputAttributes<'a>,
    }

    impl<'a> BoundOutputs<'a> {}

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
        pub fn bind(self) -> BoundInputs<'a> {
            BoundInputs { atts: self }
        }
    }

    pub struct BoundInputs<'a> {
        atts: InputAttributes<'a>,
    }

    impl<'a> BoundInputs<'a> {
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
            let attributes =
                unsafe { (*node.ptr).declare_attributes(node.ptr, declared_node.init_info) };

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
                unsafe {
                    (*n.ptr).evaluate();
                }
            }
        }
    }
}
