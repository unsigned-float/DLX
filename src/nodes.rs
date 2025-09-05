//! Node definitions.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};

static NODE_COUNT: AtomicUsize = AtomicUsize::new(0);

type RcNode = Rc<RefCell<Node>>;
type WeakNode = Weak<RefCell<Node>>;

/// The type of nodes used by the solver.
#[derive(Clone)]
pub struct Node {
    u: WeakNode,
    d: WeakNode,
    l: WeakNode,
    r: WeakNode,
    c: WeakNode,
    data: usize,
    id: usize,
}
impl Node {
    /// Create a new node with data.
    pub fn new(data: usize) -> RcNode {
        Rc::new_cyclic(|n| RefCell::new(Node {
            u: n.clone(),
            d: n.clone(),
            l: n.clone(),
            r: n.clone(),
            c: n.clone(),
            data,
            id: NODE_COUNT.fetch_add(1, Ordering::Relaxed),
        }))
    }

    /// Unlink a node horizontally by node.L.R ← node.R, node.R.L ← node.L
    fn unlink_lr(node: &RcNode) {
        let l = weak2rc(&node.borrow().l);
        let r = weak2rc(&node.borrow().r);

        l.borrow_mut().r = Rc::downgrade(&r);
        r.borrow_mut().l = Rc::downgrade(&l);
    }

    /// Unlink a node vertically by node.U.D ← node.D, node.D.U ← node.U
    fn unlink_ud(node: &RcNode) {
        let u = weak2rc(&node.borrow().u);
        let d = weak2rc(&node.borrow().d);

        u.borrow_mut().d = Rc::downgrade(&d);
        d.borrow_mut().u = Rc::downgrade(&u);
    }

    /// Relink a node horizontally by node.L.R ← node, node.R.L ← node
    fn link_lr(node: &RcNode) {
        let l = weak2rc(&node.borrow().l);
        let r = weak2rc(&node.borrow().r);

        l.borrow_mut().r = Rc::downgrade(&node);
        r.borrow_mut().l = Rc::downgrade(&node);
    }

    /// Relink a node vertically by node.U.D ← node, node.D.U ← node
    fn link_ud(node: &RcNode) {
        let u = weak2rc(&node.borrow().u);
        let d = weak2rc(&node.borrow().d);

        u.borrow_mut().d = Rc::downgrade(&node);
        d.borrow_mut().u = Rc::downgrade(&node);
    }

    /// Build a structure of nodes from a bool matrix, returning the root node.
    pub fn build(input: &Vec<Vec<bool>>) -> (RcNode, Vec<RcNode>) {
        let width = input[0].len();

        let root = Node::new(0);
        let headers: Vec<RcNode> = (0..width).map(|_| Node::new(0)).collect();
        let mut all_nodes = headers.clone();

        root.borrow_mut().r = Rc::downgrade(&headers[0]);
        headers[0].borrow_mut().l = Rc::downgrade(&root);

        root.borrow_mut().l = Rc::downgrade(&headers[width - 1]);
        headers[width - 1].borrow_mut().r = Rc::downgrade(&root);

        for i in 0..width {
            if i != 0 { headers[i].borrow_mut().l = Rc::downgrade(&headers[i - 1]); }
            if i != width - 1 { headers[i].borrow_mut().r = Rc::downgrade(&headers[i + 1]); }
        }

        for (y, row) in input.iter().enumerate() {
            let mut row_nodes = Vec::with_capacity(width);

            for (x, val) in row.iter().enumerate() {
                if *val {
                    let node = Node::new(y);
                    node.borrow_mut().c = Rc::downgrade(&headers[x]);
                    node.borrow_mut().d = Rc::downgrade(&headers[x]);
                    node.borrow_mut().u = headers[x].borrow().u.clone();

                    { weak2rc(&headers[x].borrow_mut().u) }.borrow_mut().d = Rc::downgrade(&node);
                    headers[x].borrow_mut().u = Rc::downgrade(&node);
                    headers[x].borrow_mut().data += 1;

                    all_nodes.push(node.clone());
                    row_nodes.push(node);
                }
            }

            let len = row_nodes.len();
            if len != 0 {
                row_nodes[0].borrow_mut().l = Rc::downgrade(&row_nodes[len - 1]);
                row_nodes[len - 1].borrow_mut().r = Rc::downgrade(&row_nodes[0]);
                for i in 0..len {
                    if i != 0 { row_nodes[i].borrow_mut().l = Rc::downgrade(&row_nodes[i - 1]); }
                    if i != len - 1 { row_nodes[i].borrow_mut().r = Rc::downgrade(&row_nodes[i + 1]); }
                }
            }
        }

        for header in headers {
            if header.borrow().data == 0 { Self::unlink_lr(&header); }
        }

        (root, all_nodes)
    }

    /// Build the node matrix from a list of objects and their variations.
    pub fn matrix_from_variations(input: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
        let variations = input.len();
        input.iter().enumerate().map(|(i, row)| {
            let mut entry = vec![false; variations];
            entry.extend(row);
            entry[i] = true;
            entry
        }).collect()
    }

    /// Cover a column.
    #[allow(unused_braces)]
    fn cover(header: &RcNode) {
        // get rid of it from col headers
        Self::unlink_lr(header);

        let start_col_id = header.borrow().id;
        let mut current_col = weak2rc(&header.borrow().d);
        let mut current_col_id = current_col.borrow().id;

        // loop through all nodes in the column
        while current_col_id != start_col_id {
            let start_node_id = current_col.borrow().id;
            let mut current_node = weak2rc(&current_col.borrow().r);
            let mut current_node_id = current_node.borrow().id;

            // loop through all nodes in this row
            while current_node_id != start_node_id {
                // remove it from its column, decrement size
                Self::unlink_ud(&current_node);
                weak2rc(&current_node.borrow().c).borrow_mut().data -= 1;

                // next node in this row
                current_node = { weak2rc(&current_node.borrow().r) };
                current_node_id = current_node.borrow().id;
            }

            // next node in the column
            current_col = { weak2rc(&current_col.borrow().d) };
            current_col_id = current_col.borrow().id;
        }
    }

    #[allow(unused_braces)]
    /// Undo the covering operation from a column.
    fn uncover(header: &RcNode) {
        // put it back into column headers
        Self::link_lr(header);

        let start_col_id = header.borrow().id;
        let mut current_col = weak2rc(&header.borrow().u);
        let mut current_col_id = current_col.borrow().id;

        // loop through all nodes in the column
        while current_col_id != start_col_id {
            let start_node_id = current_col.borrow().id;
            let mut current_node = weak2rc(&current_col.borrow().l);
            let mut current_node_id = current_node.borrow().id;

            // loop through all nodes in this row
            while current_node_id != start_node_id {
                Self::link_ud(&current_node);
                weak2rc(&current_node.borrow().c).borrow_mut().data += 1;

                // next node in this row
                current_node = { weak2rc(&current_node.borrow().l) };
                current_node_id = current_node.borrow().id;
            }

            // next node in the column
            current_col = { weak2rc(&current_col.borrow().u) };
            current_col_id = current_col.borrow().id;
        }
    }

    /// Search all solutions from the root node using the DLX algorithm.
    #[allow(unused_braces)]
    fn search_all(root: &RcNode, all_nodes: &Vec<RcNode>, solution: &mut Vec<usize>, partial_results: &mut Vec<Vec<usize>>) {
        let root_id = root.borrow_mut().id;
        if { weak2rc(&root.borrow().r) }.borrow().id == root_id {
            partial_results.push(solution.clone());
            return;
        }

        let mut current_node = weak2rc(&root.borrow().r);
        let mut current_node_id = current_node.borrow().id;
        let mut best_col = None;
        let mut min_size = usize::MAX;

        // find the column with the smallest amount of ones
        while current_node_id != root_id {
            let current_size = current_node.borrow().data;
            if current_size < min_size {
                min_size = current_size;
                best_col = Some(Rc::downgrade(&current_node));
            }

            current_node = { weak2rc(&current_node.borrow().r) };
            current_node_id = current_node.borrow().id;
        }

        let best_col = best_col.unwrap().upgrade().unwrap();

        Self::cover(&best_col);

        // loop through all rows that have a one in this column
        let start_row_id = best_col.borrow().id;
        let mut current_row = weak2rc(&best_col.borrow().d);
        let mut current_row_id = current_row.borrow().id;
        while current_row_id != start_row_id {
            solution.push(current_row.borrow().data);

            // loop through all columns intersecting with this row
            let start_node_id = current_row.borrow().id;
            let mut current_node = weak2rc(&current_row.borrow().r);
            let mut current_node_id = current_node.borrow().id;
            while current_node_id != start_node_id {
                // cover it
                Self::cover(&weak2rc(&current_node.borrow().c));

                // next intersecting column
                current_node = { weak2rc(&current_node.borrow().r) };
                current_node_id = current_node.borrow().id;
            }

            Self::search_all(root, &all_nodes, solution, partial_results);

            // backtracking: loop through all columns intersecting with this row
            let start_node_id = current_row.borrow().id;
            let mut current_node = weak2rc(&current_row.borrow().l);
            let mut current_node_id = current_node.borrow().id;
            while current_node_id != start_node_id {
                // cover it
                Self::uncover(&weak2rc(&current_node.borrow().c));

                // next intersecting column
                current_node = { weak2rc(&current_node.borrow().l) };
                current_node_id = current_node.borrow().id;
            }

            solution.pop();

            // next row that has a one in the column
            current_row = { weak2rc(&current_row.borrow().d) };
            current_row_id = current_row.borrow().id;
        }

        Self::uncover(&best_col);
    }

    /// Solve the exact cover problem from a starting Node, finding all solutions returning indices.
    pub fn solve_all(input: &Vec<Vec<bool>>) -> Vec<Vec<usize>> {
        let (root, all_nodes) = Self::build(input);
        let mut results = Vec::new();
        Self::search_all(&root, &all_nodes, &mut Vec::new(), &mut results);
        results
    }

    /// Search one solution from the root node using the DLX algorithm.
    #[allow(unused_braces)]
    fn search_once(root: &RcNode, all_nodes: &Vec<RcNode>, solution: &mut Vec<usize>) -> Option<Vec<usize>> {
        let root_id = root.borrow_mut().id;
        if { weak2rc(&root.borrow().r) }.borrow().id == root_id { return Some(solution.clone()) }

        let mut current_node = weak2rc(&root.borrow().r);
        let mut current_node_id = current_node.borrow().id;
        let mut best_col = None;
        let mut min_size = usize::MAX;

        // find the column with the smallest amount of ones
        while current_node_id != root_id {
            let current_size = current_node.borrow().data;
            if current_size < min_size {
                min_size = current_size;
                best_col = Some(Rc::downgrade(&current_node));
            }

            current_node = { weak2rc(&current_node.borrow().r) };
            current_node_id = current_node.borrow().id;
        }

        let best_col = best_col.unwrap().upgrade().unwrap();

        Self::cover(&best_col);

        // loop through all rows that have a one in this column
        let start_row_id = best_col.borrow().id;
        let mut current_row = weak2rc(&best_col.borrow().d);
        let mut current_row_id = current_row.borrow().id;
        while current_row_id != start_row_id {
            solution.push(current_row.borrow().data);

            // loop through all columns intersecting with this row
            let start_node_id = current_row.borrow().id;
            let mut current_node = weak2rc(&current_row.borrow().r);
            let mut current_node_id = current_node.borrow().id;
            while current_node_id != start_node_id {
                // cover it
                Self::cover(&weak2rc(&current_node.borrow().c));

                // next intersecting column
                current_node = { weak2rc(&current_node.borrow().r) };
                current_node_id = current_node.borrow().id;
            }

            if let Some(solution) = Self::search_once(root, &all_nodes, solution) { return Some(solution); }

            // backtracking: loop through all columns intersecting with this row
            let start_node_id = current_row.borrow().id;
            let mut current_node = weak2rc(&current_row.borrow().l);
            let mut current_node_id = current_node.borrow().id;
            while current_node_id != start_node_id {
                // cover it
                Self::uncover(&weak2rc(&current_node.borrow().c));

                // next intersecting column
                current_node = { weak2rc(&current_node.borrow().l) };
                current_node_id = current_node.borrow().id;
            }

            solution.pop();

            // next row that has a one in the column
            current_row = { weak2rc(&current_row.borrow().d) };
            current_row_id = current_row.borrow().id;
        }

        Self::uncover(&best_col);
        None
    }

    /// Solve the exact cover problem from a starting Node, finding one solution returning indices.
    pub fn solve_once(input: &Vec<Vec<bool>>) -> Option<Vec<usize>> {
        let (root, all_nodes) = Self::build(input);
        Self::search_once(&root, &all_nodes, &mut Vec::new())
    }
}

/// Stupid helper function... data structures in rust 😔
fn weak2rc(weak: &WeakNode) -> RcNode { weak.upgrade().unwrap() }
