use rand::random;
use std::cmp::Ordering;
use std::marker::PhantomData;
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

#[derive(Debug)]
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

    fn set_count(&self, cnt: usize) {
        unsafe { (*self.0.as_ptr()).cnt = cnt }
    }

    fn set_size(&self, size: usize) {
        unsafe { (*self.0.as_ptr()).siz = size }
    }

    fn value(&self) -> &T {
        unsafe { &(*self.0.as_ptr()).value }
    }

    fn prior(&self) -> u64 {
        unsafe { (*self.0.as_ptr()).prior }
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
        let mut res = self.count();
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
pub struct Treap<T: Ord> {
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
    /// Creates an empty tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let tree: Treap<i32> = Treap::new();
    /// assert!(tree.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            root: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Returns the number of nodes in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// assert_eq!(tree.len(), 0);
    ///
    /// tree.insert(1);
    /// tree.insert(1);
    /// assert_eq!(tree.len(), 1);
    ///
    /// tree.insert(2);
    /// assert_eq!(tree.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the tree contains no nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// assert!(tree.is_empty());
    ///
    /// tree.insert(1);
    /// assert!(!tree.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of **values** (as this is a multiset) in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// assert_eq!(tree.size(), 0);
    ///
    /// tree.insert(1);
    /// tree.insert(1);
    /// assert_eq!(tree.size(), 2);
    ///
    /// tree.insert(2);
    /// assert_eq!(tree.size(), 3);
    /// ```
    pub fn size(&self) -> usize {
        self.root.map_or(0, |root| root.size())
    }

    /// Returns an iterator over all value-number pairs in ascending value order.
    ///
    /// The iterator borrows the tree and yields value-number pairs `(&T, usize)`. It is also a
    /// [`DoubleEndedIterator`], so it can be reversed with [`Iterator::rev`].
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// tree.insert(3);
    /// tree.insert(3);
    /// tree.insert(1);
    /// tree.insert(2);
    ///
    /// let forward: Vec<_> = tree.iter().map(|(v, cnt)| (*v, cnt)).collect();
    /// assert_eq!(forward, vec![(1, 1), (2, 1), (3, 2)]);
    ///
    /// let backward: Vec<_> = tree.iter().rev().map(|(v, cnt)| (*v, cnt)).collect();
    /// assert_eq!(backward, vec![(3, 2), (2, 1), (1, 1)]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            head: self.root.map(|x| x.min_node()),
            tail: self.root.map(|x| x.max_node()),
            len: self.len(),
            _marker: PhantomData,
        }
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

    /// Returns `true` if the tree contains the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// assert!(!tree.contains(&1));
    ///
    /// tree.insert(1);
    /// assert!(tree.contains(&1));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.find_node(value).is_some()
    }

    /// Returns the number of the value in the tree.
    ///
    /// Returns `0` if the tree does not contain the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// assert_eq!(tree.count(&1), 0);
    ///
    /// tree.insert(1);
    /// tree.insert(1);
    /// assert_eq!(tree.count(&1), 2);
    /// ```
    pub fn count(&self, value: &T) -> usize {
        self.find_node(value).map_or(0, |node| node.count())
    }

    /// Returns a `Option<&T>`, the minimum value in the tree,
    /// `None` if the tree does not contain the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// tree.insert(1);
    /// tree.insert(2);
    /// tree.insert(3);
    ///
    /// assert_eq!(*tree.first().unwrap(), 1);
    /// ```
    pub fn first(&self) -> Option<&T> {
        unsafe { Some(self.root?.min_node().value_ref()) }
    }

    /// Returns a `Option<&T>`, the maximum value in the tree,
    /// `None` if the tree does not contain the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// tree.insert(1);
    /// tree.insert(2);
    /// tree.insert(3);
    ///
    /// assert_eq!(*tree.last().unwrap(), 3);
    /// ```
    pub fn last(&self) -> Option<&T> {
        unsafe { Some(self.root?.max_node().value_ref()) }
    }

    fn rotate_toward(&mut self, node: NodePtr<T>, dir: Dir) {
        let Some(far) = node.child(dir.opposite()) else {
            return;
        };
        let far_near = far.child(dir);
        let parent = node.parent();

        far.set_parent(parent);
        if let Some(parent) = parent {
            parent.set_child(node.dir_from_parent().unwrap(), Some(far));
            parent.pull();
        } else {
            self.root = Some(far);
        }

        node.set_parent(Some(far));
        far.set_child(dir, Some(node));

        far_near.map(|far_near| far_near.set_parent(Some(node)));
        node.set_child(dir.opposite(), far_near);

        node.pull();
        far.pull();
    }

    fn rotate_left(&mut self, node: NodePtr<T>) {
        self.rotate_toward(node, Dir::Left);
    }

    fn rotate_right(&mut self, node: NodePtr<T>) {
        self.rotate_toward(node, Dir::Right);
    }

    /// Insert a value into treap.
    ///
    /// Returns the number of the value in the treap after the insertion.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// assert_eq!(tree.insert(1), 1);
    /// assert_eq!(tree.insert(1), 2);
    /// assert_eq!(tree.insert(2), 1);
    /// assert!(tree.contains(&1));
    /// assert!(tree.contains(&2));
    /// assert!(!tree.contains(&3));
    /// ```
    pub fn insert(&mut self, value: T) -> usize {
        let mut last = None;
        let mut p = self.root;
        while let Some(node) = p {
            last = Some(node);
            match value.cmp(node.value()) {
                Ordering::Less => p = node.left(),
                Ordering::Greater => p = node.right(),
                Ordering::Equal => {
                    let cnt = node.count();
                    node.set_count(cnt + 1);
                    node.pull();
                    return cnt + 1;
                }
            }
        }

        let new_node = NodePtr::new(value);
        if let Some(last) = last {
            let parent = last;
            new_node.set_parent(Some(parent.clone()));
            if new_node.value() < parent.value() {
                parent.set_left(Some(new_node));
            } else {
                parent.set_right(Some(new_node));
            }
        } else {
            self.root = Some(new_node);
        }

        while let Some(parent) = new_node.parent() {
            if new_node.prior() < parent.prior() {
                self.rotate_toward(parent, new_node.dir_from_parent().unwrap().opposite());
            } else {
                break;
            }
        }

        self.len += 1;

        1
    }

    fn move_to_leaf(&mut self, node: NodePtr<T>) {
        loop {
            match (node.left(), node.right()) {
                (Some(left), Some(right)) => {
                    if left.prior() < right.prior() {
                        self.rotate_right(node);
                    } else {
                        self.rotate_left(node);
                    }
                }
                (Some(_), None) => {
                    self.rotate_right(node);
                }
                (None, Some(_)) => {
                    self.rotate_left(node);
                }
                (None, None) => {
                    if let Some(parent) = node.parent() {
                        parent.set_child(node.dir_from_parent().unwrap(), None);
                    } else {
                        self.root = None;
                    }
                    break;
                }
            }
        }
    }

    fn release(node: NodePtr<T>) {
        unsafe {
            let node = Box::from_raw(node.0.as_ptr());
            drop(node);
        }
    }

    /// Remove one of the value in the treap.
    ///
    /// Returns `Option<usize>`, the number of value after deletion, `None` if the value didn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// tree.insert(1);
    /// tree.insert(1);
    /// assert_eq!(tree.remove_one_of(&1), Some(1));
    /// assert_eq!(tree.remove_one_of(&1), Some(0));
    /// assert_eq!(tree.remove_all_of(&1), None);
    /// ```
    pub fn remove_one_of(&mut self, value: &T) -> Option<usize> {
        let node = self.find_node(value)?;
        let cnt = node.count() - 1;
        node.set_count(cnt);

        if cnt == 0 {
            self.move_to_leaf(node);
            Self::release(node);
        }

        Some(cnt)
    }

    /// Remove all values of the value in the treap.
    ///
    /// Returns `Option<usize>`, 0 if the value existed in the tree, `None` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// tree.insert(1);
    /// assert_eq!(tree.remove_all_of(&1), Some(0));
    /// assert_eq!(tree.remove_all_of(&1), None);
    /// ```
    pub fn remove_all_of(&mut self, value: &T) -> Option<usize> {
        let node = self.find_node(value)?;
        self.move_to_leaf(node);
        Self::release(node);
        return Some(0);
    }

    /// A alias of `remove_all_of`, be careful when you just want to remove one of the values.
    pub fn remove(&mut self, value: &T) -> Option<usize> {
        self.remove_all_of(value)
    }

    /// Removes all nodes from the tree.
    ///
    /// The tree remains usable after it is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// use treap::Treap;
    ///
    /// let mut tree = Treap::new();
    /// tree.insert(1);
    /// tree.insert(2);
    ///
    /// tree.clear();
    ///
    /// assert!(tree.is_empty());
    /// assert!(!tree.contains(&1));
    /// ```
    pub fn clear(&mut self) {
        let mut stack = Vec::new();
        if let Some(root) = self.root.take() {
            stack.push(root);
        }

        while let Some(node) = stack.pop() {
            if let Some(left) = node.left() {
                stack.push(left);
            }
            if let Some(right) = node.right() {
                stack.push(right);
            }
            unsafe {
                drop(Box::from_raw(node.0.as_ptr()));
            }
        }

        self.len = 0;
    }
}

impl<T: Ord> Drop for Treap<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// An iterator over borrowed entries in an [`Treap`].
///
/// This iterator is created by [`Treap::iter`]. It yields values in
/// ascending order and can also iterate from the back.
pub struct Iter<'a, T: Ord + 'a> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Ord + 'a> Iterator for Iter<'a, T> {
    type Item = (&'a T, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let p = self.head?;
        self.len -= 1;

        if self.len == 0 {
            self.head = None;
            self.tail = None;
        } else {
            self.head = p.next();
        }

        unsafe {
            let v = &p.0.as_ref().value;
            let cnt = p.count();
            Some((v, cnt))
        }
    }
}

impl<'a, T: Ord + 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let p = self.tail?;
        self.len -= 1;

        if self.len == 0 {
            self.head = None;
            self.tail = None;
        } else {
            self.tail = p.prev();
        }

        unsafe {
            let v = &p.0.as_ref().value;
            let cnt = p.count();
            Some((v, cnt))
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, TestCaseError};
    use std::collections::{BTreeMap, HashSet, VecDeque};

    fn model_insert(model: &mut BTreeMap<i32, usize>, x: i32) -> usize {
        let cnt = model.entry(x).or_insert(0);
        *cnt += 1;
        *cnt
    }

    fn model_remove_one(model: &mut BTreeMap<i32, usize>, x: i32) -> Option<usize> {
        let cnt = model.get_mut(&x)?;
        *cnt -= 1;
        let rest = *cnt;

        if rest == 0 {
            model.remove(&x);
        }

        Some(rest)
    }

    fn model_remove_all(model: &mut BTreeMap<i32, usize>, x: i32) -> Option<usize> {
        model.remove(&x).map(|_| 0)
    }

    fn model_size(model: &BTreeMap<i32, usize>) -> usize {
        model.values().sum()
    }

    fn model_items(model: &BTreeMap<i32, usize>) -> Vec<(i32, usize)> {
        model.iter().map(|(&v, &cnt)| (v, cnt)).collect()
    }

    fn check_subtree(
        node: Link<i32>,
        parent: Link<i32>,
        lower: Option<i32>,
        upper: Option<i32>,
        seen: &mut HashSet<usize>,
    ) -> Result<(usize, usize), TestCaseError> {
        let Some(node) = node else {
            return Ok((0, 0));
        };

        let addr = node.0.as_ptr() as usize;
        prop_assert!(seen.insert(addr), "cycle detected");

        prop_assert_eq!(node.parent(), parent);

        let value = *node.value();

        if let Some(lower) = lower {
            prop_assert!(lower < value);
        }

        if let Some(upper) = upper {
            prop_assert!(value < upper);
        }

        prop_assert!(node.count() > 0);

        if let Some(left) = node.left() {
            prop_assert_eq!(left.parent(), Some(node));
            prop_assert!(node.prior() <= left.prior());
        }

        if let Some(right) = node.right() {
            prop_assert_eq!(right.parent(), Some(node));
            prop_assert!(node.prior() <= right.prior());
        }

        let (left_len, left_size) = check_subtree(
            node.left(),
            Some(node),
            lower,
            Some(value),
            seen,
        )?;

        let (right_len, right_size) = check_subtree(
            node.right(),
            Some(node),
            Some(value),
            upper,
            seen,
        )?;

        let expected_size = node.count() + left_size + right_size;

        prop_assert_eq!(node.size(), expected_size);

        Ok((1 + left_len + right_len, expected_size))
    }

    fn assert_internal_invariants(treap: &Treap<i32>) -> Result<(), TestCaseError> {
        let mut seen = HashSet::new();

        if let Some(root) = treap.root {
            prop_assert_eq!(root.parent(), None);
        }

        let (node_count, total_size) =
            check_subtree(treap.root, None, None, None, &mut seen)?;

        prop_assert_eq!(node_count, treap.len());
        prop_assert_eq!(total_size, treap.size());

        Ok(())
    }

    fn assert_public_state(
        treap: &Treap<i32>,
        model: &BTreeMap<i32, usize>,
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(treap.len(), model.len());
        prop_assert_eq!(treap.size(), model_size(model));
        prop_assert_eq!(treap.is_empty(), model.is_empty());

        prop_assert_eq!(
            treap.first().copied(),
            model.keys().next().copied(),
        );

        prop_assert_eq!(
            treap.last().copied(),
            model.keys().next_back().copied(),
        );

        prop_assert_eq!(
            treap.iter().map(|(v, cnt)| (*v, cnt)).collect::<Vec<_>>(),
            model_items(model),
        );

        let mut backward = model_items(model);
        backward.reverse();

        prop_assert_eq!(
            treap.iter().rev().map(|(v, cnt)| (*v, cnt)).collect::<Vec<_>>(),
            backward,
        );

        for x in -105..=105 {
            prop_assert_eq!(treap.contains(&x), model.contains_key(&x));
            prop_assert_eq!(treap.count(&x), *model.get(&x).unwrap_or(&0));
        }

        assert_internal_invariants(treap)?;

        Ok(())
    }

    proptest! {
        #![proptest_config(Config {
            cases: 256,
            max_shrink_iters: 10_000,
            .. ProptestConfig::default()
        })]

        #[test]
        fn treap_matches_btreemap_multiset(
            ops in proptest::collection::vec((0u8..5, -100i32..100), 0..1000),
        ) {
            let mut treap = Treap::new();
            let mut model = BTreeMap::new();

            for (op, x) in ops {
                match op {
                    0 => {
                        let expected = model_insert(&mut model, x);
                        prop_assert_eq!(treap.insert(x), expected);
                    }
                    1 => {
                        let expected = model_remove_one(&mut model, x);
                        prop_assert_eq!(treap.remove_one_of(&x), expected);
                    }
                    2 => {
                        let expected = model_remove_all(&mut model, x);
                        prop_assert_eq!(treap.remove_all_of(&x), expected);
                    }
                    3 => {
                        let expected = model_remove_all(&mut model, x);
                        prop_assert_eq!(treap.remove(&x), expected);
                    }
                    _ => {
                        prop_assert_eq!(treap.contains(&x), model.contains_key(&x));
                        prop_assert_eq!(treap.count(&x), *model.get(&x).unwrap_or(&0));
                    }
                }

                assert_public_state(&treap, &model)?;
            }
        }

        #[test]
        fn duplicate_insert_and_remove_one_counts_are_correct(
            xs in proptest::collection::vec(-20i32..20, 0..500),
        ) {
            let mut treap = Treap::new();
            let mut model = BTreeMap::new();

            for &x in &xs {
                let expected = model_insert(&mut model, x);
                prop_assert_eq!(treap.insert(x), expected);
                prop_assert_eq!(treap.count(&x), expected);
                assert_public_state(&treap, &model)?;
            }

            for &x in &xs {
                let expected = model_remove_one(&mut model, x);
                prop_assert_eq!(treap.remove_one_of(&x), expected);
                assert_public_state(&treap, &model)?;
            }
        }

        #[test]
        fn clear_resets_tree_and_tree_remains_reusable(
            xs in proptest::collection::vec(-100i32..100, 0..500),
            ys in proptest::collection::vec(-100i32..100, 0..500),
        ) {
            let mut treap = Treap::new();
            let mut model = BTreeMap::new();

            for x in xs {
                let expected = model_insert(&mut model, x);
                prop_assert_eq!(treap.insert(x), expected);
            }

            assert_public_state(&treap, &model)?;

            treap.clear();
            model.clear();

            assert_public_state(&treap, &model)?;

            for y in ys {
                let expected = model_insert(&mut model, y);
                prop_assert_eq!(treap.insert(y), expected);
                assert_public_state(&treap, &model)?;
            }
        }

        #[test]
        fn double_ended_iterator_matches_model(
            xs in proptest::collection::vec(-50i32..50, 0..300),
            choices in proptest::collection::vec(any::<bool>(), 0..400),
        ) {
            let mut treap = Treap::new();
            let mut model = BTreeMap::new();

            for x in xs {
                model_insert(&mut model, x);
                treap.insert(x);
            }

            assert_public_state(&treap, &model)?;

            let mut expected: VecDeque<_> = model_items(&model).into_iter().collect();
            let mut iter = treap.iter();

            for take_front in choices {
                let got = if take_front {
                    iter.next()
                } else {
                    iter.next_back()
                }
                .map(|(v, cnt)| (*v, cnt));

                let expected_item = if take_front {
                    expected.pop_front()
                } else {
                    expected.pop_back()
                };

                prop_assert_eq!(got, expected_item);
            }

            let rest = iter.map(|(v, cnt)| (*v, cnt)).collect::<Vec<_>>();
            let expected_rest = expected.into_iter().collect::<Vec<_>>();

            prop_assert_eq!(rest, expected_rest);
        }

        #[test]
        fn first_last_are_consistent_after_random_updates(
            ops in proptest::collection::vec((0u8..4, -100i32..100), 0..1000),
        ) {
            let mut treap = Treap::new();
            let mut model = BTreeMap::new();

            for (op, x) in ops {
                match op {
                    0 => {
                        model_insert(&mut model, x);
                        treap.insert(x);
                    }
                    1 => {
                        model_remove_one(&mut model, x);
                        treap.remove_one_of(&x);
                    }
                    2 => {
                        model_remove_all(&mut model, x);
                        treap.remove_all_of(&x);
                    }
                    _ => {
                        prop_assert_eq!(treap.contains(&x), model.contains_key(&x));
                    }
                }

                prop_assert_eq!(treap.first().copied(), model.keys().next().copied());
                prop_assert_eq!(treap.last().copied(), model.keys().next_back().copied());

                assert_public_state(&treap, &model)?;
            }
        }
    }
}