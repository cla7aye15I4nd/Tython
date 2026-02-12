class Counter:
    val: int
    def __init__(self, start: int) -> None:
        self.val = start
    def inc(self) -> None:
        self.val += 1
    def add(self, n: int) -> None:
        self.val += n
    def mul(self, n: int) -> None:
        self.val *= n

def test_aug_assign_field_inc() -> None:
    c: Counter = Counter(0)
    c.inc()
    c.inc()
    c.inc()
    assert c.val == 3
    print("aug_field_inc ok")

def test_aug_assign_field_add() -> None:
    c: Counter = Counter(10)
    c.add(5)
    c.add(3)
    assert c.val == 18
    print("aug_field_add ok")

def test_aug_assign_field_mul() -> None:
    c: Counter = Counter(2)
    c.mul(3)
    c.mul(4)
    assert c.val == 24
    print("aug_field_mul ok")

def run_tests() -> None:
    test_aug_assign_field_inc()
    test_aug_assign_field_add()
    test_aug_assign_field_mul()
