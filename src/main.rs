extern crate comp_graph;
use comp_graph::compute_graph::{Output, Input, ComputationalNode, DeclaredNode, Graph, GraphBuilder, NodeAttributes};


struct Node1 {
    output1: Output<f64>,
}

impl Node1 {
    fn new() -> DeclaredNode {
        let mut node = Box::new(Node1{output1: Default::default()});
        let mut attributes = NodeAttributes::new();
        attributes.add_output("x".to_string(), &node.output1);
        DeclaredNode{node, attributes}
    }
}


impl ComputationalNode for Node1 {
    fn evaluate(&mut self) {
        *self.output1 += 1.0
    }
}

struct Node2 {
    input1: Input<f64>,
}

impl Node2 {
    fn new(input_name: String) -> DeclaredNode {
        let mut node = Box::new(Node2{input1: Default::default()});
        let mut attributes = NodeAttributes::new();
        attributes.add_input(input_name, &mut node.input1);
        DeclaredNode{node, attributes}
    }
}

impl ComputationalNode for Node2 {
    fn evaluate(&mut self) {
        println!("{}", *self.input1)
    }
}


fn main() {
    let mut builder = GraphBuilder::new();
    builder.add("hello".to_string(), Node1::new());
    builder.add("goodbye".to_string(), Node2::new("hello.x".to_string()));
    let mut graph = builder.build();
    graph.evaluate();
    graph.evaluate();
    graph.evaluate();
}