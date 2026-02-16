class Passive:
    pass


class WithPassAndField:
    pass
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def get(self) -> int:
        return self.value


def test_class_body_pass_statement() -> None:
    x: WithPassAndField = WithPassAndField(42)
    print("CHECK test_class_pass_body lhs:", x.get())
    print("CHECK test_class_pass_body rhs:", 42)
    assert x.get() == 42


def run_tests() -> None:
    test_class_body_pass_statement()


if __name__ == "__main__":
    run_tests()
