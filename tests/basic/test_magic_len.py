class Bag:
    count: int

    def __init__(self, count: int) -> None:
        self.count = count

    def __len__(self) -> int:
        return self.count


def test_len_on_class_magic() -> None:
    b: Bag = Bag(4)
    n: int = len(b)
    print(n)
    assert n == 4


def run_tests() -> None:
    test_len_on_class_magic()
