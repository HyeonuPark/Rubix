use std::rc::Rc;

use router::Router;

#[derive(Debug)]
pub struct Session {
    router: Rc<Router>,
}
