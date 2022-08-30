#![feature(strict_provenance)]

pub mod compute_graph {
    use std::any::Any;
    use std::collections::HashMap;
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use std::ptr;

    pub struct AliasBox<T: ?Sized> {
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
    }

    trait InputSetter: 'static {
        fn set(&mut self, target: *const dyn Any) -> Result<(), ()>;
    }

    impl<T: 'static> InputSetter for Input<T> {
        fn set(&mut self, target: *const dyn Any) -> Result<(), ()> {
            unsafe { &*target }.is::<T>().then_some(()).ok_or(())?;
            self.data = target.cast();
            Ok(())
        }
    }

    pub struct NodeAttributes {
        inputs: HashMap<String, *mut dyn InputSetter>,
        outputs: HashMap<String, *mut dyn Any>,
    }

    impl NodeAttributes {
        pub unsafe fn new(
            input_attrs: InputAttributes,
            output_attrs: OutputAttributes,
            attrs: &Attributes,
        ) -> Self {
            NodeAttributes {
                inputs: attrs.inputs.transform(input_attrs.data),
                outputs: attrs.outputs.transform(output_attrs.data),
            }
        }
    }

    pub struct DeclaredNode {
        node: AliasBox<dyn UnsafeNode + 'static>,
        attrs: NodeAttributes,
    }

    impl DeclaredNode {
        pub unsafe fn new(
            node: AliasBox<dyn UnsafeNode + 'static>,
            attrs: NodeAttributes,
        ) -> DeclaredNode {
            DeclaredNode { node, attrs }
        }
    }

    pub struct GraphBuilder {
        nodes: Vec<AliasBox<dyn UnsafeNode>>,
        inputs: HashMap<String, Vec<*mut dyn InputSetter>>,
        outputs: HashMap<String, *const dyn Any>,
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
            for (key, value) in declared_node.attrs.inputs {
                if !self.inputs.contains_key(&key) {
                    self.inputs.insert(key.clone(), Vec::new());
                }
                self.inputs.get_mut(&key).unwrap().push(value);
            }
            for (key, value) in declared_node.attrs.outputs {
                self.outputs.insert(format!("{name}.{key}"), value);
            }

            self.nodes.push(declared_node.node);
        }
        pub fn build(self) -> Graph {
            for (key, value) in self.inputs {
                let output_lookup = *self
                    .outputs
                    .get(&key)
                    .unwrap_or_else(|| panic!("Error, no output for input {}", key));
                for input_setter in value {
                    unsafe { &mut *input_setter }
                        .set(output_lookup)
                        .unwrap_or_else(|_| {
                            panic!("Error, input and output types mismatch at {}", key)
                        });
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

    // Structs that help in safetly implementing nodes
    pub struct OutputAttributes<'a> {
        data: HashMap<String, *mut dyn Any>,
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
            let input: *const T = &output.data;
            let input: *mut T = self.master_ptr.with_addr(input.addr()).cast();
            self.data.insert(name.to_string(), input);
        }
    }

    pub struct InputAttributes<'a> {
        data: HashMap<String, *mut dyn InputSetter>,
        phantom: PhantomData<&'a dyn Any>,
    }

    impl<'a> InputAttributes<'a> {
        pub fn new() -> Self {
            InputAttributes {
                data: HashMap::new(),
                phantom: PhantomData,
            }
        }
        pub fn add<T: 'static>(&mut self, name: &str, input: &'a mut Input<T>) {
            self.data.insert(name.to_string(), input);
        }
    }
    // These traits to be implemented automatically and safely by Derive macros
    pub unsafe trait OutputStruct {
        fn declare_outputs<'a>(&'a self, outputs: &mut OutputAttributes<'a>);
    }

    pub unsafe trait InputStruct {
        fn new(_: InputMaker) -> Self;
        fn declare_inputs<'a>(&'a mut self, inputs: &mut InputAttributes<'a>);
    }

    // Currently, only supports renames but should also support dropping attributes
    #[derive(Default)]
    pub struct AttributeTransformer {
        data: HashMap<String, String>,
    }

    impl AttributeTransformer {
        pub fn rename(&mut self, old_name: &str, new_name: &str) {
            self.data.insert(old_name.to_string(), new_name.to_string());
        }

        fn transform<V>(&self, mut attrs: HashMap<String, V>) -> HashMap<String, V> {
            for (old_name, new_name) in &self.data {
                let (_, v) = attrs.remove_entry(old_name).unwrap();
                attrs.insert(new_name.to_string(), v);
            }

            attrs
        }
    }

    #[derive(Default)]
    pub struct Attributes {
        pub inputs: AttributeTransformer,
        pub outputs: AttributeTransformer,
    }

    // Safe trait, most users just implement this
    pub trait ComputationalNode {
        type Outputs: OutputStruct;
        type Inputs: InputStruct;
        type InitInfo;

        fn make(init_info: Self::InitInfo, attrs: &mut Attributes) -> (Self, Self::Outputs)
        where
            Self: Sized;

        fn evaluate(&mut self, inputs: &Self::Inputs, outputs: &mut Self::Outputs);
    }

    pub struct ErasedNode<T: ComputationalNode> {
        node: T,
        inputs: T::Inputs,
        outputs: T::Outputs,
    }

    unsafe impl<T: ComputationalNode> UnsafeNode for ErasedNode<T> {
        fn evaluate(&mut self) {
            self.node.evaluate(&self.inputs, &mut self.outputs);
        }
    }

    pub trait ComputationalNodeMaker {
        type InitInfo;
        fn declare(init_info: Self::InitInfo) -> DeclaredNode;
    }

    impl<T: ComputationalNode + 'static> ComputationalNodeMaker for T {
        type InitInfo = T::InitInfo;
        fn declare(init_info: Self::InitInfo) -> DeclaredNode {
            let inputs = T::Inputs::new(InputMaker {
                phantom: PhantomData,
            });
            let mut attrs = Attributes::default();
            let (node, outputs) = T::make(init_info, &mut attrs);
            let node = AliasBox::new(Box::new(ErasedNode {
                node,
                inputs,
                outputs,
            }));
            let mut input_attrs = InputAttributes::new();
            let mut output_attrs = OutputAttributes::new(node.ptr);
            let attrs = unsafe {
                (*node.ptr).inputs.declare_inputs(&mut input_attrs);
                (*node.ptr).outputs.declare_outputs(&mut output_attrs);
                NodeAttributes::new(input_attrs, output_attrs, &attrs)
            };
            let p = node.ptr;
            std::mem::forget(node);
            unsafe {
                DeclaredNode::new(
                    AliasBox {
                        ptr: p,
                        phantom: PhantomData,
                    },
                    attrs,
                )
            }
        }
    }
}
