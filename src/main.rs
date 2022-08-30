use comp_graph::compute_graph::{
    Attributes, ComputationalNode, ComputationalNodeMaker, DeclaredNode, Graph, GraphBuilder,
    Input, InputMaker, Output,
};
use comp_graph_macro::{InputStruct, OutputStruct};
use serde::Deserialize;
use serde_json::Value;

use std::marker::PhantomData;

use std::collections::HashMap;
use std::path::Path;
#[derive(Default)]
struct JsonNodeFactory {
    registry: HashMap<String, fn(Value) -> DeclaredNode>,
}

impl JsonNodeFactory {
    fn make(&self, name: &str, v: Value) -> DeclaredNode {
        self.registry.get(name).unwrap()(v)
    }
    fn new() -> Self {
        let mut factory = JsonNodeFactory::default();

        for reg_node in inventory::iter::<RegisteredNode> {
            factory
                .registry
                .insert(reg_node.name.to_string(), reg_node.declarer);
        }

        factory
    }
}

struct RegisteredNode {
    name: &'static str,
    declarer: fn(Value) -> DeclaredNode,
}

impl RegisteredNode {
    const fn new<T: ComputationalNode + 'static>(name: &'static str) -> Self
    where
        T::InitInfo: for<'a> Deserialize<'a>,
    {
        RegisteredNode {
            name,
            declarer: |v| {
                let init_info: T::InitInfo = serde_json::from_value(v).unwrap();
                T::declare(init_info)
            },
        }
    }
}

inventory::collect!(RegisteredNode);

#[derive(Default, OutputStruct)]
struct Node1Outputs {
    x: Output<f64>,
    y: Output<f64>,
}

#[derive(InputStruct)]
struct Node1Inputs {}

#[derive(Deserialize)]
struct Node1InitInfo {}

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

inventory::submit! {
    RegisteredNode::new::<Node1>("Node1")
}

#[derive(OutputStruct)]
struct PrinterOutputs {}

#[derive(Deserialize)]
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
inventory::submit! {
    RegisteredNode::new::<Printer<f64>>("Printer<f64>")
}

#[derive(Deserialize)]
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
inventory::submit! {
    RegisteredNode::new::<Multiplier>("Multiplier")
}

fn graph_from_json(path: &Path) -> Graph {
    let node_factory = JsonNodeFactory::new();

    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let j: Vec<Value> = serde_json::from_reader(reader).unwrap();

    let mut builder = GraphBuilder::new();
    for mut value in j.into_iter() {
        let obj_ref = value.as_object_mut().unwrap();
        let node_type = obj_ref
            .remove("__type__")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let node_name = obj_ref
            .remove("__name__")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        builder.add(&node_name, node_factory.make(&node_type, value));
    }

    builder.build()
}

fn graph_from_code() -> Graph {
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

    builder.build()
}


fn main() {
    let mut graph = graph_from_code();

    graph.evaluate();
    graph.evaluate();
    graph.evaluate();
}
