extern crate comp_graph;
use comp_graph::compute_graph::{Output, Input, ComputationalNode, DeclaredNode, Graph, GraphBuilder, NodeAttributes,
    InputStruct, OutputStruct, InputAttributes, OutputAttributes, InputMaker, UnsafeNode, BoundInputs, BoundOutputs};

use std::marker::PhantomData;

#[derive(Default)]
struct Node1Outputs {
    output1: Output<f64>,
    output2: Output<f64>,
}

unsafe impl OutputStruct for Node1Outputs {
    fn declare_outputs<'a>(&'a self, outputs: &mut OutputAttributes<'a>) {
        outputs.add("x", &self.output1);
        outputs.add("y", &self.output2);
    }
}

struct Node1Inputs;

unsafe impl InputStruct for Node1Inputs {
    fn new(_: InputMaker) -> Self {
        Node1Inputs {}
    }
    fn declare_inputs<'a>(&'a mut self, _inputs: &mut InputAttributes<'a>) {}
}

struct Node1InitInfo;

struct Node1;

impl ComputationalNode for Node1 {
    type Outputs = Node1Outputs;
    type Inputs = Node1Inputs;
    type InitInfo = Node1InitInfo;

    fn initialize(
        &mut self,
        _init_info: Self::InitInfo,
        _bound_inputs: BoundInputs,
        _bound_outputs: BoundOutputs,
    ) {
    }

    fn evaluate(&mut self, _inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        *outputs.output1 += 1.0;
        *outputs.output2 += 2.0;
    }
}

fn make_node1() -> DeclaredNode {
    DeclaredNode::new(Node1 {}, Node1InitInfo {}, Default::default())
}

struct PrinterOutputs;

unsafe impl OutputStruct for PrinterOutputs {
    fn declare_outputs<'a>(&'a self, _outputs: &'a mut OutputAttributes) {}
}

struct PrinterInitInfo {
    input_name: String,
}

struct PrinterInputs<T> {
    input: Input<T>,
}

unsafe impl<T: 'static> InputStruct for PrinterInputs<T> {
    fn new(i: InputMaker) -> Self {
        PrinterInputs {
            input: Input::new(i),
        }
    }
    fn declare_inputs<'a>(&'a mut self, inputs: &mut InputAttributes<'a>) {
        inputs.add("input", &mut self.input);
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

    fn initialize(
        &mut self,
        init_info: Self::InitInfo,
        mut bound_inputs: BoundInputs,
        _bound_outputs: BoundOutputs,
    ) {
        bound_inputs.rename("input", &init_info.input_name);
    }

    fn evaluate(&mut self, inputs: &Self::Inputs, _outputs: &mut Self::Outputs) {
        println!("Printing: {}, input: {}", self.print_prefix, *inputs.input);
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
        PrinterOutputs {},
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

struct MultiplierInputs {
    input1: Input<f64>,
    input2: Input<f64>,
}

struct Multiplier;

impl ComputationalNode for Multiplier {
    type InitInfo = MultiplierInitInfo;
    type Inputs = MultiplierInputs;
    type Outputs = MultiplierOutputs;

    fn initialize(
        &mut self,
        init_info: Self::InitInfo,
        mut bound_inputs: BoundInputs,
        _bound_outputs: BoundOutputs,
    ) {
        bound_inputs.rename("input1", &init_info.input1_name);
        bound_inputs.rename("input2", &init_info.input2_name);
    }

    fn evaluate(&mut self, inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        *outputs.product = *inputs.input1 * *inputs.input2;
    }
}

fn make_multiplier(input1_name: String, input2_name: String) -> DeclaredNode {
    DeclaredNode::new(
        Multiplier {},
        MultiplierInitInfo {
            input1_name,
            input2_name,
        },
        Default::default(),
    )
}

unsafe impl OutputStruct for MultiplierOutputs {
    fn declare_outputs<'a>(&'a self, outputs: &mut OutputAttributes<'a>) {
        outputs.add("product", &self.product);
    }
}

unsafe impl InputStruct for MultiplierInputs {
    fn new(i: InputMaker) -> Self {
        MultiplierInputs {
            input1: Input::new(i),
            input2: Input::new(i),
        }
    }
    fn declare_inputs<'a>(&'a mut self, inputs: &mut InputAttributes<'a>) {
        inputs.add("input1", &mut self.input1);
        inputs.add("input2", &mut self.input2);
    }
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