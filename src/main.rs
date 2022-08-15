extern crate comp_graph;
use comp_graph::compute_graph::{Output, Input, ComputationalNode, DeclaredNode, Graph, GraphBuilder, NodeAttributes, InputStruct, OutputStruct, InputAttributes, OutputAttributes, InputMaker};

use std::marker::PhantomData;

#[derive(Default)]
struct Node1Outputs {
    output1: Output<f64>,
    output2: Output<f64>,
}

unsafe impl OutputStruct for Node1Outputs {
    fn declare_outputs(&self) -> OutputAttributes {
        let mut outputs = OutputAttributes::new();
        outputs.add("x".to_string(), &self.output1);
        outputs.add("y".to_string(), &self.output2);
        outputs
    }
}

struct Node1Inputs;

unsafe impl InputStruct for Node1Inputs {
    fn new(_: InputMaker) -> Self {
        Node1Inputs {}
    }
}

struct Node1InitInfo;

struct Node1;

impl ComputationalNode for Node1 {
    type Outputs = Node1Outputs;
    type Inputs = Node1Inputs;
    type InitInfo = Node1InitInfo;

    fn create_outputs(&mut self, init_info: &Self::InitInfo) -> Self::Outputs {
        Default::default()
    }

    fn initialize<'a>(
        &mut self,
        init_info: Self::InitInfo,
        inputs: &'a Self::Inputs,
    ) -> InputAttributes<'a> {
        InputAttributes::new()
    }

    fn evaluate(&mut self, inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        *outputs.output1.get_mut() += 1.0;
        *outputs.output2.get_mut() += 2.0;
    }
}

fn make_node1() -> DeclaredNode {
    DeclaredNode::new(Node1 {}, Node1InitInfo {})
}

struct PrinterOutputs;

unsafe impl OutputStruct for PrinterOutputs {
    fn declare_outputs(&self) -> OutputAttributes {
        OutputAttributes::new()
    }
}

struct PrinterInitInfo {
    input_name: String,
}

struct PrinterInputs<T> {
    input: Input<T>,
}

unsafe impl<T> InputStruct for PrinterInputs<T> {
    fn new(i: InputMaker) -> Self {
        PrinterInputs {
            input: Input::new(i),
        }
    }
}

struct Printer<T: std::fmt::Display> {
    print_prefix: String,
    phantom: PhantomData<T>,
}

impl<T: std::fmt::Display + 'static> ComputationalNode for Printer<T> {
    type Inputs = PrinterInputs<T>;
    type Outputs = PrinterOutputs;
    type InitInfo = PrinterInitInfo;

    fn initialize<'a>(
        &mut self,
        init_info: Self::InitInfo,
        inputs: &'a Self::Inputs,
    ) -> InputAttributes<'a> {
        let mut input_atts = InputAttributes::new();
        input_atts.add(init_info.input_name, &inputs.input);
        input_atts
    }

    fn evaluate(&mut self, inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        println!(
            "Printing: {}, input: {}",
            self.print_prefix,
            *inputs.input.get()
        );
    }

    fn create_outputs(&mut self, init_info: &Self::InitInfo) -> Self::Outputs {
        PrinterOutputs {}
    }
}

fn make_printer<T: std::fmt::Display + 'static>(
    input_name: String,
    print_prefix: String,
) -> DeclaredNode {
    DeclaredNode::new(
        Printer::<T> {
            print_prefix,
            phantom: PhantomData,
        },
        PrinterInitInfo { input_name },
    )
}

struct MultiplierInitInfo {
    input1_name: String,
    input2_name: String,
}

#[derive(Default)]
struct MultiplierOutputs {
    product: Output<f64>,
}

unsafe impl OutputStruct for MultiplierOutputs {
    fn declare_outputs(&self) -> OutputAttributes {
        let mut out_atts = OutputAttributes::new();
        out_atts.add("product".to_string(), &self.product);
        out_atts
    }
}

struct MultiplierInputs {
    input1: Input<f64>,
    input2: Input<f64>,
}

unsafe impl InputStruct for MultiplierInputs {
    fn new(i: InputMaker) -> Self {
        MultiplierInputs {
            input1: Input::new(i),
            input2: Input::new(i),
        }
    }
}

struct Multiplier;

impl ComputationalNode for Multiplier {
    type InitInfo = MultiplierInitInfo;
    type Inputs = MultiplierInputs;
    type Outputs = MultiplierOutputs;

    fn initialize<'a>(
        &mut self,
        init_info: Self::InitInfo,
        inputs: &'a Self::Inputs,
    ) -> InputAttributes<'a> {
        let mut input_atts = InputAttributes::new();
        input_atts.add(init_info.input1_name, &inputs.input1);
        input_atts.add(init_info.input2_name, &inputs.input2);
        input_atts
    }

    fn evaluate(&mut self, inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        *outputs.product.get_mut() = *inputs.input1.get() * (*inputs.input2.get());
    }

    fn create_outputs(&mut self, init_info: &Self::InitInfo) -> Self::Outputs {
        Default::default()
    }
}

fn make_multiplier(input1_name: String, input2_name: String) -> DeclaredNode {
    DeclaredNode::new(
        Multiplier {},
        MultiplierInitInfo {
            input1_name,
            input2_name,
        },
    )
}

fn main() {
    let mut builder = GraphBuilder::new();
    builder.add("start", make_node1());
    builder.add(
        "print_x",
        make_printer::<f64>("start.x".to_string(), "x".to_string()),
    );
    builder.add(
        "print_y",
        make_printer::<f64>("start.y".to_string(), "y".to_string()),
    );
    builder.add(
        "product",
        make_multiplier("start.x".to_string(), "start.y".to_string()),
    );
    builder.add(
        "print_product",
        make_printer::<f64>("product.product".to_string(), "product".to_string()),
    );

    let mut graph = builder.build();
    graph.evaluate();
    graph.evaluate();
    graph.evaluate();
}