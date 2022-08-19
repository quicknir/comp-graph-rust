#![feature(strict_provenance)]

pub mod compute_graph {
    use core::mem::MaybeUninit;
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

    // This trait to be implemented via macro
    pub unsafe trait AttributeStruct {
        fn new(_: InputMaker) -> Self;
        fn declare_attributes<'a>(&'a mut self, atts: &mut NodeAttributes<'a>);
    }

    // Safe trait, most users just implement this
    pub trait ComputationalNode {
        type Attributes: AttributeStruct;
        type InitInfo;

        fn new(
            init_info: Self::InitInfo,
            bound_attrs: BoundAttributes,
            attrs: &mut Self::Attributes,
        ) -> Self;

        fn evaluate(&mut self, attrs: &mut Self::Attributes);
    }

    pub struct ErasedNode<T: ComputationalNode + 'static> {
        attrs: T::Attributes,
        node: MaybeUninit<T>,
    }

    unsafe impl<T: ComputationalNode + 'static> UnsafeNode for ErasedNode<T> {
        fn evaluate(&mut self) {
            unsafe { &mut *self.node.as_mut_ptr() }.evaluate(&mut self.attrs);
        }
    }

    impl<T: ComputationalNode + 'static> Drop for ErasedNode<T> {
        fn drop(&mut self) {
            unsafe { self.node.assume_init_drop() };
        }
    }

    pub fn declare_node<T: ComputationalNode + 'static>(init_info: T::InitInfo) -> DeclaredNode {
        let e = ErasedNode::<T> {
            attrs: T::Attributes::new(InputMaker {
                phantom: PhantomData,
            }),
            node: MaybeUninit::uninit(),
        };
        let p = Box::into_raw(Box::new(e));
        let mut attrs = NodeAttributes::new(p);
        unsafe {&mut *p} .attrs.declare_attributes(&mut attrs);
        let bound_attrs = attrs.bind();
        unsafe {
            (*p).node = MaybeUninit::new(T::new(init_info, bound_attrs, &mut (*p).attrs));
            DeclaredNode::new (
                AliasBox::new(Box::from_raw(p)),
                attrs)
        }
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

    pub struct BoundAttributes<'a, 'b> {
        atts: &'b mut NodeAttributes<'a>,
    }
    impl BoundAttributes<'_, '_> {
        pub fn rename_input(&mut self, old_name: &str, new_name: &str) {
            let (_, input) = self.atts.inputs.remove_entry(old_name).unwrap();
            self.atts.inputs.insert(new_name.to_string(), input);
        }
        pub fn rename_output(&mut self, old_name: &str, new_name: &str) {
            let (_, input) = self.atts.outputs.remove_entry(old_name).unwrap();
            self.atts.outputs.insert(new_name.to_string(), input);
        }
    }

    pub struct NodeAttributes<'a> {
        inputs: HashMap<String, *mut dyn InputSetter>,
        outputs: HashMap<String, *mut dyn Any>,
        phantom: PhantomData<&'a dyn Any>,
        master_ptr: *mut u8,
    }

    impl<'a> NodeAttributes<'a> {
        pub fn new(master_ptr: *mut dyn UnsafeNode) -> Self {
            NodeAttributes { 
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                phantom: PhantomData,
                master_ptr: master_ptr.cast(),
            }
        }
        pub fn add_input<T: 'static>(&mut self, name: &str, input: &'a mut Input<T>) {
            let mut input_ptr: *mut Input<T> = input;
            input_ptr = self.master_ptr.with_addr(input_ptr.addr()).cast();
            self.inputs.insert(name.to_string(), input_ptr);
        }
        pub fn add_output<T: Any + 'static>(&mut self, name: &str, output: &'a Output<T>) {
            let output_ptr: *const T = &output.data;
            let output_ptr: *mut T = self.master_ptr.with_addr(output_ptr.addr()).cast();
            self.outputs.insert(name.to_string(), output_ptr);
        }

        pub fn bind<'b>(&'b mut self) -> BoundAttributes<'a, 'b> {
            BoundAttributes { atts: self }
        }
    }

    pub struct DeclaredNode {
        node: AliasBox<dyn UnsafeNode + 'static>,
        inputs: HashMap<String, *mut dyn InputSetter>,
        outputs: HashMap<String, *mut dyn Any>,
    }

    impl DeclaredNode {
        pub unsafe fn new(
            node: AliasBox<dyn UnsafeNode + 'static>,
            attrs: NodeAttributes,
        ) -> DeclaredNode {
            DeclaredNode {
                node,
                inputs: attrs.inputs,
                outputs: attrs.outputs,
            }
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
            for (key, value) in declared_node.inputs {
                if !self.inputs.contains_key(&key) {
                    self.inputs.insert(key.clone(), Vec::new());
                }
                self.inputs.get_mut(&key).unwrap().push(value);
            }
            for (key, value) in declared_node.outputs {
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
}
