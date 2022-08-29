extern crate comp_graph;
extern crate comp_graph_macro;

use comp_graph::compute_graph::{
    Attributes, ComputationalNode, ComputationalNodeMaker, GraphBuilder, Input, InputMaker, Output,
};
use comp_graph_macro::{InputStruct, OutputStruct};

use std::marker::PhantomData;

#[derive(Default, OutputStruct)]
struct Node1Outputs {
    x: Output<f64>,
    y: Output<f64>,
}

#[derive(InputStruct)]
struct Node1Inputs {}

struct Node1InitInfo;

struct Node1;

impl ComputationalNode for Node1 {
    type Outputs = Node1Outputs;
    type Inputs = Node1Inputs;
    type InitInfo = Node1InitInfo;

    fn make(_init_info: Self::InitInfo, _attrs: &mut Attributes) -> (Self, Self::Outputs) {
        (Node1 {}, Default::default())
    }

    fn evaluate(&mut self, _inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        *outputs.x += 1.0;
        *outputs.y += 2.0;
    }
}

#[derive(OutputStruct)]
struct PrinterOutputs {}

struct PrinterInitInfo {
    print_prefix: String,
    input_name: String,
}

#[derive(InputStruct)]
struct PrinterInputs<T: 'static> {
    input: Input<T>,
}

struct Printer<T: std::fmt::Display> {
    print_prefix: String,
    phantom: PhantomData<T>,
}

impl<T: std::fmt::Display + 'static> ComputationalNode for Printer<T> {
    type Inputs = PrinterInputs<T>;
    type Outputs = PrinterOutputs;
    type InitInfo = PrinterInitInfo;

    fn make(init_info: Self::InitInfo, attrs: &mut Attributes) -> (Self, Self::Outputs) {
        attrs.inputs.rename("input", &init_info.input_name);
        (
            Printer {
                print_prefix: init_info.print_prefix,
                phantom: PhantomData,
            },
            PrinterOutputs {},
        )
    }
    fn evaluate(&mut self, inputs: &Self::Inputs, _outputs: &mut Self::Outputs) {
        println!("Printing: {}, input: {}", self.print_prefix, *inputs.input);
    }
}

struct MultiplierInitInfo {
    input1_name: String,
    input2_name: String,
}

#[derive(Default, OutputStruct)]
struct MultiplierOutputs {
    product: Output<f64>,
}

#[derive(InputStruct)]
struct MultiplierInputs {
    input1: Input<f64>,
    input2: Input<f64>,
}

struct Multiplier;

impl ComputationalNode for Multiplier {
    type InitInfo = MultiplierInitInfo;
    type Inputs = MultiplierInputs;
    type Outputs = MultiplierOutputs;

    fn make(init_info: Self::InitInfo, attrs: &mut Attributes) -> (Self, Self::Outputs) {
        attrs.inputs.rename("input1", &init_info.input1_name);
        attrs.inputs.rename("input2", &init_info.input2_name);
        (Multiplier {}, Default::default())
    }

    fn evaluate(&mut self, inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        *outputs.product = *inputs.input1 * *inputs.input2;
    }
}

fn main() {
    let mut builder = GraphBuilder::new();
    builder.add("start", Node1::declare(Node1InitInfo {}));
    builder.add(
        "print_x",
        Printer::<f64>::declare(PrinterInitInfo {
            input_name: "start.x".to_string(),
            print_prefix: "x".to_string(),
        }),
    );
    builder.add(
        "print_y",
        Printer::<f64>::declare(PrinterInitInfo {
            input_name: "start.y".to_string(),
            print_prefix: "y".to_string(),
        }),
    );
    builder.add(
        "product",
        Multiplier::declare(MultiplierInitInfo {
            input1_name: "start.x".to_string(),
            input2_name: "start.y".to_string(),
        }),
    );
    builder.add(
        "print_product",
        Printer::<f64>::declare(PrinterInitInfo {
            input_name: "product.product".to_string(),
            print_prefix: "product".to_string(),
        }),
    );

    let mut graph = builder.build();
    graph.evaluate();
    graph.evaluate();
    graph.evaluate();
}
