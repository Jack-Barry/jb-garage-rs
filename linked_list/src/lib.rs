#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }
}

pub fn reverse_list(mut node: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    let mut prev: Option<Box<ListNode>> = None;

    while let Some(mut n) = node {
        let next = n.next.take();
        n.next = prev.take();
        prev = Some(n);
        node = next;
    }

    prev
}

#[cfg(test)]
mod tests {
    use crate::{reverse_list, ListNode};

    #[test]
    fn reverse_list_reverses_list() {
        let node_3 = ListNode::new(3);
        let node_2 = ListNode {
            val: 2,
            next: Some(Box::new(node_3)),
        };
        let node_1 = ListNode {
            val: 1,
            next: Some(Box::new(node_2)),
        };

        let new_head = reverse_list(Some(Box::new(node_1)));
        let new_node_1 = new_head.as_ref().unwrap();
        let new_node_2 = new_node_1.next.as_ref().unwrap();
        let new_node_3 = new_node_2.next.as_ref().unwrap();
        assert_eq!(new_node_1.val, 3);
        assert_eq!(new_node_2.val, 2);
        assert_eq!(new_node_3.val, 1);
    }
}
