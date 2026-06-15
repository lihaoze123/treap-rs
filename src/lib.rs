use std::cmp::Ordering;
use std::marker::PhantomData;
use rand::random;
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Left,
    Right,
}

impl Dir {
    fn opposite(self) -> Self {
        match self {
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }

    fn index(self) -> usize {
        match self {
            Dir::Left => 0,
            Dir::Right => 1,
        }
    }
}

type Link<T> = Option<NodePtr<T>>;

struct Node<T> {
    parent: Link<T>,
    children: [Link<T>; 2],
    value: T,
    prior: u64,
    cnt: usize,
    siz: usize,
}

struct NodePtr<T>(NonNull<Node<T>>);

impl<T> Clone for NodePtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for NodePtr<T> {}

impl<T> PartialEq for NodePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            parent: None,
            children: [None; 2],
            value,
            prior: random::<u64>(),
            cnt: 1,
            siz: 1,
        }
    }
}

impl<T> NodePtr<T> {
    fn new(value: T) -> Self {
        Self(NonNull::from(Box::leak(Box::new(Node::new(value)))))
    }

    fn count(&self) -> usize {
        unsafe { (*self.0.as_ptr()).cnt }
    }

    fn size(&self) -> usize {
        unsafe { (*self.0.as_ptr()).siz }
    }

    fn set_cnt(&self, cnt: usize) {
        unsafe { (*self.0.as_ptr()).cnt = cnt }
    }

    fn set_size(&self, size: usize) {
        unsafe { (*self.0.as_ptr()).siz = size }
    }

    fn value(&self) -> &T {
        unsafe { &(*self.0.as_ptr()).value }
    }

    unsafe fn value_ref<'a>(&self) -> &'a T {
        unsafe { &(*self.0.as_ptr()).value }
    }

    fn parent(&self) -> Option<NodePtr<T>> {
        unsafe { (*self.0.as_ptr()).parent }
    }

    fn child(&self, dir: Dir) -> Link<T> {
        unsafe { (*self.0.as_ptr()).children[dir.index()] }
    }

    fn set_parent(&self, parent: Link<T>) {
        unsafe { (*self.0.as_ptr()).parent = parent }
    }

    fn set_child(&self, dir: Dir, child: Link<T>) {
        unsafe { (*self.0.as_ptr()).children[dir.index()] = child }
    }

    fn set_left(&self, child: Link<T>) {
        self.set_child(Dir::Left, child);
    }

    fn set_right(&self, child: Link<T>) {
        self.set_child(Dir::Right, child);
    }

    fn set_prior(&self, prior: u64) {
        unsafe { (*self.0.as_ptr()).prior = prior }
    }

    fn left(&self) -> Link<T> {
        self.child(Dir::Left)
    }

    fn right(&self) -> Link<T> {
        self.child(Dir::Right)
    }

    fn size_of(node: Link<T>) -> usize {
        match node {
            None => 0,
            Some(node) => node.size(),
        }
    }

    fn pull(&self) {
        let mut res = 1;
        res += Self::size_of(self.left());
        res += Self::size_of(self.right());
        self.set_size(res);
    }

    fn dir_from_parent(self) -> Option<Dir> {
        let parent = self.parent()?;
        if parent.child(Dir::Left) == Some(self) {
            Some(Dir::Left)
        } else {
            Some(Dir::Right)
        }
    }
}

impl<T: Ord> NodePtr<T> {
    fn minmax_node(&self, dir: Dir) -> NodePtr<T> {
        let mut p = *self;
        while let Some(node) = p.child(dir) {
            p = node;
        }
        p
    }

    fn min_node(&self) -> NodePtr<T> {
        self.minmax_node(Dir::Left)
    }

    fn max_node(&self) -> NodePtr<T> {
        self.minmax_node(Dir::Right)
    }

    fn neighbor(&self, dir: Dir) -> Link<T> {
        if let Some(node) = self.child(dir) {
            return Some(node.minmax_node(dir.opposite()));
        }
        let mut p = *self;
        while let Some(parent) = p.parent() {
            if p.dir_from_parent() == Some(dir.opposite()) {
                return Some(parent);
            }
            p = parent;
        }
        None
    }

    fn prev(&self) -> Link<T> {
        self.neighbor(Dir::Left)
    }

    fn next(&self) -> Link<T> {
        self.neighbor(Dir::Right)
    }
}

/// An ordered set implemented with a treap.
///
/// values are kept in sorted order according to their [`Ord`] implementation.
pub struct Treap<T> {
    root: Link<T>,
    len: usize,
    _marker: PhantomData<Box<Node<T>>>,
}

unsafe impl<T: Ord + Send> Send for Treap<T> {}
unsafe impl<T: Ord + Sync> Sync for Treap<T> {}

impl<T: Ord> Default for Treap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Treap<T> {
    pub fn new() -> Self {
        Self {
            root: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn find_node(&self, value: &T) -> Link<T> {
        let mut p = self.root;
        while let Some(node) = p {
            match value.cmp(node.value()) {
                Ordering::Less => p = node.left(),
                Ordering::Greater => p = node.right(),
                Ordering::Equal => return Some(node),
            }
        }
        None
    }

    pub fn contains(&self, value: &T) -> bool {
        self.find_node(value).is_some()
    }

    pub fn first(&self) -> Option<&T> {
        unsafe { Some(self.root?.min_node().value_ref()) }
    }

    pub fn last(&self) -> Option<&T> {
        unsafe { Some(self.root?.max_node().value_ref()) }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
