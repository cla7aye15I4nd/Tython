class BinarySearchTree:
    values: list[int]
    left: list[int]
    right: list[int]
    root: int

    def __init__(self) -> None:
        self.values = []
        self.left = []
        self.right = []
        self.root = -1

    def _new_node(self, value: int) -> int:
        self.values.append(value)
        self.left.append(-1)
        self.right.append(-1)
        return len(self.values) - 1

    def insert(self, key: int) -> None:
        if self.root == -1:
            self.root = self._new_node(key)
            return

        cur: int = self.root
        while True:
            if key < self.values[cur]:
                if self.left[cur] == -1:
                    self.left[cur] = self._new_node(key)
                    break
                cur = self.left[cur]
            else:
                if self.right[cur] == -1:
                    self.right[cur] = self._new_node(key)
                    break
                cur = self.right[cur]

    def inorder_iterative(self) -> list[int]:
        result: list[int] = []
        stack: list[int] = []
        cur: int = self.root

        while cur != -1 or len(stack) > 0:
            while cur != -1:
                stack.append(cur)
                cur = self.left[cur]

            cur = stack.pop()
            result.append(self.values[cur])
            cur = self.right[cur]
        return result

    def height_bfs(self) -> int:
        if self.root == -1:
            return 0

        q: list[int] = [self.root]
        head: int = 0
        level_count: int = 1
        next_count: int = 0
        height: int = 0

        while head < len(q):
            node: int = q[head]
            head = head + 1
            level_count = level_count - 1

            if self.left[node] != -1:
                q.append(self.left[node])
                next_count = next_count + 1
            if self.right[node] != -1:
                q.append(self.right[node])
                next_count = next_count + 1

            if level_count == 0:
                height = height + 1
                level_count = next_count
                next_count = 0

        return height

    def lca(self, a: int, b: int) -> int:
        lo: int = a
        hi: int = b
        if lo > hi:
            t: int = lo
            lo = hi
            hi = t

        cur: int = self.root
        while cur != -1:
            v: int = self.values[cur]
            if hi < v:
                cur = self.left[cur]
            elif lo > v:
                cur = self.right[cur]
            else:
                return v
        return -1

    def has_path_sum(self, target: int) -> bool:
        if self.root == -1:
            return False

        node_stack: list[int] = [self.root]
        sum_stack: list[int] = [self.values[self.root]]
        while len(node_stack) > 0:
            node: int = node_stack.pop()
            acc: int = sum_stack.pop()

            is_leaf: bool = self.left[node] == -1 and self.right[node] == -1
            if is_leaf and acc == target:
                return True

            if self.right[node] != -1:
                r: int = self.right[node]
                node_stack.append(r)
                sum_stack.append(acc + self.values[r])
            if self.left[node] != -1:
                l: int = self.left[node]
                node_stack.append(l)
                sum_stack.append(acc + self.values[l])
        return False


def test_binary_tree_bst_class_algorithms() -> None:
    bst: BinarySearchTree = BinarySearchTree()
    data: list[int] = [8, 3, 10, 1, 6, 14, 4, 7, 13, 6, 7, 14]
    for x in data:
        bst.insert(x)

    inorder: list[int] = bst.inorder_iterative()
    print('CHECK test_tree_algorithms lhs:', inorder)
    print('CHECK test_tree_algorithms rhs:', [1, 3, 4, 6, 6, 7, 7, 8, 10, 13, 14, 14])
    assert inorder == [1, 3, 4, 6, 6, 7, 7, 8, 10, 13, 14, 14]

    print('CHECK test_tree_algorithms lhs:', bst.height_bfs())
    print('CHECK test_tree_algorithms rhs:', 5)
    assert bst.height_bfs() == 5

    print('CHECK test_tree_algorithms lhs:', bst.lca(1, 7))
    print('CHECK test_tree_algorithms rhs:', 3)
    assert bst.lca(1, 7) == 3
    print('CHECK test_tree_algorithms lhs:', bst.lca(4, 14))
    print('CHECK test_tree_algorithms rhs:', 8)
    assert bst.lca(4, 14) == 8
    print('CHECK test_tree_algorithms lhs:', bst.lca(13, 14))
    print('CHECK test_tree_algorithms rhs:', 14)
    assert bst.lca(13, 14) == 14

    print('CHECK test_tree_algorithms assert expr:', 'bst.has_path_sum(12)')
    assert bst.has_path_sum(12)  # 8 -> 3 -> 1
    print('CHECK test_tree_algorithms assert expr:', 'bst.has_path_sum(31)')
    assert bst.has_path_sum(31)  # 8 -> 10 -> 13
    print('CHECK test_tree_algorithms assert expr:', 'not bst.has_path_sum(100)')
    assert not bst.has_path_sum(100)

    weighted: int = 0
    i: int = 0
    while i < len(inorder):
        weighted = weighted + inorder[i] * (i + 1)
        i = i + 1
    print('CHECK test_tree_algorithms lhs:', weighted)
    print('CHECK test_tree_algorithms rhs:', 770)
    assert weighted == 770


def run_tests() -> None:
    test_binary_tree_bst_class_algorithms()
