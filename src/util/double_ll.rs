/*
 * Create doubly linked list using unsafe. Currently these lists are only
 * used in LRU caches for the chunks, which are static and do not use
 * allocation. Therefore, in the current case, use of unsafe here is valid.
 */
#[derive(Clone)]
pub struct LinkedNode<T> {
    value: T,
    prev: Option<*mut LinkedNode<T>>,
    next: Option<*mut LinkedNode<T>>,
}

pub struct DoublyLinkedList<T> {
    head: Option<*mut LinkedNode<T>>,
    tail: Option<*mut LinkedNode<T>>,
    size: usize,
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self {
            head: None,
            tail: None,
            size: 0,
        }
    }
}

impl<T> DoublyLinkedList<T> {
    pub fn push_back(&mut self, value: T) {
        // if the list already has items:
        if let Some(tail) = self.tail {
            // then we create a new node and append it to the back.
            let mut node = Box::new(LinkedNode {
                value,
                prev: Some(tail),
                next: None,
            });
            let node = Box::into_raw(node);
            unsafe {
                (*tail).next = Some(node);
            }
            self.tail = Some(node);
        } else {
            // otherwise if the list is empty, we add a new node which
            // is the value of both head and tail.
            let node = Box::new(LinkedNode {
                value,
                prev: None,
                next: None,
            });
            let node_ptr = Box::into_raw(node);

            self.head = Some(node_ptr);
            self.tail = Some(node_ptr);
        }
        self.size += 1;
    }
}
