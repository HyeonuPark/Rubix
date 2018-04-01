use std::collections::HashMap;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Router {
    root: RefCell<Node>,
}

#[derive(Debug)]
struct Node {}
