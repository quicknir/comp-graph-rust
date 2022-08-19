extern crate comp_graph;
use comp_graph::compute_graph::{Output, Input, ComputationalNode, GraphBuilder, NodeAttributes,
    AttributeStruct, InputMaker, declare_node};

use std::marker::PhantomData;

#[derive(Default)]
struct Node1Attributes {
    output1: Output<f64>,
    output2: Output<f64>,
}

unsafe impl AttributeStruct for Node1Attributes {
    fn declare_attributes<'a>(&'a mut self, atts: &mut NodeAttributes<'a>) {
        atts.add_output("x", &self.output1);
        atts.add_output("y", &self.output2);
    }
    fn new(_: InputMaker) -> Self {
        Default::default()
    }
}

struct Node1InitInfo;

struct Node1;

impl ComputationalNode for Node1 {
    type Attributes = Node1Attributes;
    type InitInfo = Node1InitInfo;

    fn new(
                _init_info: Self::InitInfo,
                _bound_attrs: comp_graph::compute_graph::BoundAttributes,
                _attrs: &mut Self::Attributes,
            ) -> Self {
                Node1{}
        
    }
    fn evaluate(&mut self, attrs: &mut Self::Attributes) {
        *attrs.output1 += 1.0;
        *attrs.output2 += 2.0;
    }
}

struct PrinterAttributes<T> {
    input: Input<T>,
}

unsafe impl<T: 'static> AttributeStruct for PrinterAttributes<T> {
    fn new(i: InputMaker) -> Self {
        PrinterAttributes { input: Input::new(i) }
    }
    fn declare_attributes<'a>(&'a mut self, atts: &mut NodeAttributes<'a>) {
        atts.add_input("input", &mut self.input);
    }

}
struct PrinterInitInfo {
    input_name: String,
    print_prefix: String,
}

struct Printer<T: std::fmt::Display> {
    print_prefix: String,
    phantom: PhantomData<T>,
}

impl<T: std::fmt::Display + 'static> ComputationalNode for Printer<T> {
    type Attributes = PrinterAttributes<T>;
    type InitInfo = PrinterInitInfo;

    fn new(
                init_info: Self::InitInfo,
                mut bound_attrs: comp_graph::compute_graph::BoundAttributes,
                _attrs: &mut Self::Attributes,
            ) -> Self {
                bound_attrs.rename_input("input", &init_info.input_name);
                Printer { print_prefix: init_info.print_prefix, phantom: PhantomData }
    }
    fn evaluate(&mut self, attrs: &mut Self::Attributes) {
        
        println!("Printing: {}, input: {}", self.print_prefix, *attrs.input);
    }
}

struct MultiplierAttributes {
    input1: Input<f64>,
    input2: Input<f64>,
    product: Output<f64>,
}

unsafe impl AttributeStruct for MultiplierAttributes {
    fn new(i: InputMaker) -> Self {
        MultiplierAttributes{input1: Input::new(i), input2: Input::new(i), product: Default::default()}
    }
    fn declare_attributes<'a>(&'a mut self, atts: &mut NodeAttributes<'a>) {
        
    atts.add_output("product", &self.product);
    atts.add_input("input1", &mut self.input1);
    atts.add_input("input2", &mut self.input2);
}
}


struct MultiplierInitInfo {
    input1_name: String,
    input2_name: String,
}

struct Multiplier;

impl ComputationalNode for Multiplier {
    type InitInfo = MultiplierInitInfo;
    type Attributes = MultiplierAttributes;

    fn new(
                init_info: Self::InitInfo,
                mut bound_attrs: comp_graph::compute_graph::BoundAttributes,
                _attrs: &mut Self::Attributes,
            ) -> Self {
        
        bound_attrs.rename_input("input1", &init_info.input1_name);
        bound_attrs.rename_input("input2", &init_info.input2_name);
        Multiplier{}
    }
    fn evaluate(&mut self, attrs: &mut Self::Attributes) {
        
        *attrs.product = *attrs.input1 * *attrs.input2;
    }
}

fn main() {
    let mut builder = GraphBuilder::new();
    builder.add("start", declare_node::<Node1>(Node1InitInfo{}));
    builder.add(
        "print_x",
        declare_node::<Printer<f64>>(PrinterInitInfo{input_name: "start.x".to_string(), print_prefix: "x".to_string()}));
    builder.add(
        "print_y",
        declare_node::<Printer<f64>>(PrinterInitInfo{input_name: "start.y".to_string(), print_prefix: "y".to_string()}));
    builder.add(
        "product",
        declare_node::<Multiplier>(MultiplierInitInfo{input1_name: "start.x".to_string(), input2_name: "start.y".to_string()}));
    builder.add(
        "print_product",
        declare_node::<Printer<f64>>(PrinterInitInfo{input_name: "product.product".to_string(), print_prefix: "product".to_string()}));

    let mut graph = builder.build();
    graph.evaluate();
    graph.evaluate();
    graph.evaluate();
}