class AVLTree:
    key: list[int]
    left: list[int]
    right: list[int]
    height: list[int]
    root: int

    def __init__(self) -> None:
        self.key = []
        self.left = []
        self.right = []
        self.height = []
        self.root = -1

    def _new_node(self, k: int) -> int:
        self.key.append(k)
        self.left.append(-1)
        self.right.append(-1)
        self.height.append(1)
        return len(self.key) - 1

    def _h(self, n: int) -> int:
        if n == -1:
            return 0
        return self.height[n]

    def _upd(self, n: int) -> None:
        hl: int = self._h(self.left[n])
        hr: int = self._h(self.right[n])
        if hl > hr:
            self.height[n] = hl + 1
        else:
            self.height[n] = hr + 1

    def _bf(self, n: int) -> int:
        return self._h(self.left[n]) - self._h(self.right[n])

    def _rot_right(self, y: int) -> int:
        x: int = self.left[y]
        t2: int = self.right[x]
        self.right[x] = y
        self.left[y] = t2
        self._upd(y)
        self._upd(x)
        return x

    def _rot_left(self, x: int) -> int:
        y: int = self.right[x]
        t2: int = self.left[y]
        self.left[y] = x
        self.right[x] = t2
        self._upd(x)
        self._upd(y)
        return y

    def _insert_node(self, n: int, k: int) -> int:
        if n == -1:
            return self._new_node(k)

        if k < self.key[n]:
            self.left[n] = self._insert_node(self.left[n], k)
        else:
            self.right[n] = self._insert_node(self.right[n], k)

        self._upd(n)
        b: int = self._bf(n)

        if b > 1 and k < self.key[self.left[n]]:
            return self._rot_right(n)

        if b < -1 and k >= self.key[self.right[n]]:
            return self._rot_left(n)

        if b > 1 and k >= self.key[self.left[n]]:
            self.left[n] = self._rot_left(self.left[n])
            return self._rot_right(n)

        if b < -1 and k < self.key[self.right[n]]:
            self.right[n] = self._rot_right(self.right[n])
            return self._rot_left(n)

        return n

    def insert(self, k: int) -> None:
        self.root = self._insert_node(self.root, k)

    def inorder(self) -> list[int]:
        out: list[int] = []
        st: list[int] = []
        cur: int = self.root
        while cur != -1 or len(st) > 0:
            while cur != -1:
                st.append(cur)
                cur = self.left[cur]
            cur = st.pop()
            out.append(self.key[cur])
            cur = self.right[cur]
        return out

    def is_balanced(self) -> bool:
        i: int = 0
        while i < len(self.key):
            bf: int = self._bf(i)
            if bf < -1 or bf > 1:
                return False
            i = i + 1
        return True


class Treap:
    key: list[int]
    pri: list[int]
    left: list[int]
    right: list[int]
    root: int

    def __init__(self) -> None:
        self.key = []
        self.pri = []
        self.left = []
        self.right = []
        self.root = -1

    def _new_node(self, k: int, p: int) -> int:
        self.key.append(k)
        self.pri.append(p)
        self.left.append(-1)
        self.right.append(-1)
        return len(self.key) - 1

    def _priority(self, k: int) -> int:
        v: int = (k * 1103515245 + 12345) & 2147483647
        return v

    def _rot_right(self, y: int) -> int:
        x: int = self.left[y]
        t2: int = self.right[x]
        self.right[x] = y
        self.left[y] = t2
        return x

    def _rot_left(self, x: int) -> int:
        y: int = self.right[x]
        t2: int = self.left[y]
        self.left[y] = x
        self.right[x] = t2
        return y

    def _insert_node(self, n: int, k: int) -> int:
        if n == -1:
            return self._new_node(k, self._priority(k))

        if k < self.key[n]:
            self.left[n] = self._insert_node(self.left[n], k)
            if self.pri[self.left[n]] < self.pri[n]:
                return self._rot_right(n)
        else:
            self.right[n] = self._insert_node(self.right[n], k)
            if self.pri[self.right[n]] < self.pri[n]:
                return self._rot_left(n)
        return n

    def insert(self, k: int) -> None:
        self.root = self._insert_node(self.root, k)

    def inorder(self) -> list[int]:
        out: list[int] = []
        st: list[int] = []
        cur: int = self.root
        while cur != -1 or len(st) > 0:
            while cur != -1:
                st.append(cur)
                cur = self.left[cur]
            cur = st.pop()
            out.append(self.key[cur])
            cur = self.right[cur]
        return out

    def check_heap_property(self) -> bool:
        i: int = 0
        while i < len(self.key):
            l: int = self.left[i]
            r: int = self.right[i]
            if l != -1 and self.pri[l] < self.pri[i]:
                return False
            if r != -1 and self.pri[r] < self.pri[i]:
                return False
            i = i + 1
        return True


def test_avl_tree_large_insertions() -> None:
    avl: AVLTree = AVLTree()
    i: int = 0
    while i < 7000:
        v: int = (i * 73 + 19) % 10007
        avl.insert(v)
        i = i + 1

    ino: list[int] = avl.inorder()
    j: int = 1
    while j < len(ino):
        assert ino[j - 1] <= ino[j]
        j = j + 1
    assert avl.is_balanced()
    assert avl._h(avl.root) < 40
    print(len(ino))


def test_treap_large_insertions() -> None:
    tr: Treap = Treap()
    i: int = 0
    while i < 8000:
        v: int = (i * 97 + 31) % 12011
        tr.insert(v)
        i = i + 1

    ino: list[int] = tr.inorder()
    j: int = 1
    while j < len(ino):
        assert ino[j - 1] <= ino[j]
        j = j + 1
    assert tr.check_heap_property()
    print(len(ino))


def run_tests() -> None:
    test_avl_tree_large_insertions()
    test_treap_large_insertions()
