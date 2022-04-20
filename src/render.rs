use lockfree::prelude::Queue;
use once_cell::sync::Lazy;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderId {
    line: u32,
    col: u32,
}

impl RenderId {
    pub fn new(a: u32, b: u32) -> Self {
        RenderId { line: a, col: b }
    }
}

#[macro_export]
macro_rules! id {
    () => {
        crate::RenderId::new(line!(), column!())
    };
}

static RENDER_QUEUE: Lazy<Queue<RenderObject>> = Lazy::new(Default::default);

enum RenderObject {}

impl RenderObject {
    fn push(self) {
        RENDER_QUEUE.push(self);
    }
}
