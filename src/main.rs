extern crate comp_graph;
use comp_graph::compute_graph::{Output, Input, ComputationalNode, DeclaredNode, Graph, GraphBuilder, NodeAttributes};

use std::marker::PhantomData;

#[derive(Default)]
struct Node1Attributes {
    output1: Output<f64>,
    output2: Output<f64>,
}

struct Node1DeclareInfo;

struct Node1;

impl ComputationalNode for Node1 {
    type Attributes = Node1Attributes;
    type DeclareInfo = Node1DeclareInfo;

    fn evaluate(&mut self, atts: &Self::Attributes) {
        *atts.output1.get_mut() += 1.0;
        *atts.output2.get_mut() += 2.0;
    }

    fn declare_attributes(atts: &Self::Attributes, declare_info: Self::DeclareInfo) -> NodeAttributes {
        let mut attributes = NodeAttributes::new();
   
        attributes.add_output("x".to_string(), &atts.output1);
        attributes.add_output("y".to_string(), &atts.output2);

        attributes
    }
}

fn make_node1() -> DeclaredNode {
    DeclaredNode::new(Node1{}, Default::default(), Node1DeclareInfo{})
}


struct PrinterDeclareInfo {
    input_name: String,
}

struct PrinterAttributes<T> {
    input: Input<T>,
}

struct Printer<T: std::fmt::Display> {
    print_prefix: String,
    phantom: PhantomData<T>,
}


impl <T: std::fmt::Display + 'static> ComputationalNode for Printer<T> {
    type Attributes = PrinterAttributes<T>;
    type DeclareInfo = PrinterDeclareInfo;

    fn evaluate(&mut self, atts: &Self::Attributes) {
        println!("Printing: {}, output: {}", self.print_prefix, *atts.input.get());
    }

    fn declare_attributes(atts: &Self::Attributes, declare_info: Self::DeclareInfo) -> NodeAttributes {
        let mut attributes = NodeAttributes::new();

        attributes.add_input(declare_info.input_name, &atts.input);
        attributes
    }
}

fn make_printer<T: std::fmt::Display + 'static>(input_name: String, print_prefix: String) -> DeclaredNode {
    DeclaredNode::new(Printer::<T>{print_prefix, phantom: PhantomData}, PrinterAttributes::<T>{input: Default::default()}, PrinterDeclareInfo{input_name})
}

struct MultiplierDeclareInfo {
    input1_name: String,
    input2_name: String,
}

#[derive(Default)]
struct MultiplierAttributes {
    input1: Input<f64>,
    input2: Input<f64>,
    product: Output<f64>,
}

struct Multiplier;

impl ComputationalNode for Multiplier {
    type Attributes = MultiplierAttributes;
    type DeclareInfo = MultiplierDeclareInfo;

    fn evaluate(&mut self, atts: &Self::Attributes) {
        *atts.product.get_mut() = *atts.input1.get() * (*atts.input2.get());
    }

    fn declare_attributes(atts: &Self::Attributes, declare_info: Self::DeclareInfo) -> NodeAttributes {
        let mut attributes = NodeAttributes::new();
        attributes.add_input(declare_info.input1_name, &atts.input1);
        attributes.add_input(declare_info.input2_name, &atts.input2);
        attributes.add_output("product".to_string(), &atts.product);
        attributes
    }
}


fn make_multiplier(input1_name: String, input2_name: String) -> DeclaredNode {
    DeclaredNode::new(Multiplier{}, Default::default(), MultiplierDeclareInfo{input1_name, input2_name})
}



fn main() {
    let mut builder = GraphBuilder::new();
    builder.add("start", make_node1());
    builder.add("print_x", make_printer::<f64>("start.x".to_string(), "x".to_string()));
    builder.add("print_y", make_printer::<f64>("start.y".to_string(), "y".to_string()));
    builder.add("product", make_multiplier("start.x".to_string(), "start.y".to_string()));
    builder.add("print_product", make_printer::<f64>("product.product".to_string(), "product".to_string()));

    let mut graph = builder.build();
    graph.evaluate();
    graph.evaluate();
    graph.evaluate();
}
