class OneStepIter:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __iter__(self) -> "OneStepIter":
        return self

    def __next__(self) -> int:
        return self.value


def test_iter_next_class_builtins() -> None:
    it: OneStepIter = iter(OneStepIter(7))
    got: int = next(it)
    print('CHECK test_builtin_iter_next lhs:', got)
    print('CHECK test_builtin_iter_next rhs:', 7)
    assert got == 7


def run_tests() -> None:
    test_iter_next_class_builtins()
