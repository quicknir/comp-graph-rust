extern crate comp_graph;
use comp_graph::compute_graph::{
    Attributes, ComputationalNode, ComputationalNodeMaker, GraphBuilder, Input, InputAttributes,
    InputMaker, InputStruct, Output, OutputAttributes, OutputStruct,
};

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

    fn make(_init_info: Self::InitInfo, _attrs: &mut Attributes) -> (Self, Self::Outputs) {
        (Node1 {}, Default::default())
    }

    fn evaluate(&mut self, _inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        *outputs.output1 += 1.0;
        *outputs.output2 += 2.0;
    }
}

struct PrinterOutputs;

unsafe impl OutputStruct for PrinterOutputs {
    fn declare_outputs<'a>(&'a self, _outputs: &'a mut OutputAttributes) {}
}

struct PrinterInitInfo {
    print_prefix: String,
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

    fn make(init_info: Self::InitInfo, attrs: &mut Attributes) -> (Self, Self::Outputs) {
        attrs.inputs.rename("input1", &init_info.input1_name);
        attrs.inputs.rename("input2", &init_info.input2_name);
        (Multiplier {}, Default::default())
    }

    fn evaluate(&mut self, inputs: &Self::Inputs, outputs: &mut Self::Outputs) {
        *outputs.product = *inputs.input1 * *inputs.input2;
    }
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
