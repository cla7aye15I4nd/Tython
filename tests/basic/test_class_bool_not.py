class Flag:
    value: bool

    def __init__(self, value: bool) -> None:
        self.value = value

    def __bool__(self) -> bool:
        return self.value


def test_not_on_class_instance() -> None:
    t: Flag = Flag(True)
    f: Flag = Flag(False)
    print("CHECK test_class_bool_not lhs:", not t)
    print("CHECK test_class_bool_not rhs:", False)
    assert (not t) == False
    print("CHECK test_class_bool_not lhs:", not f)
    print("CHECK test_class_bool_not rhs:", True)
    assert (not f) == True


def run_tests() -> None:
    test_not_on_class_instance()


if __name__ == "__main__":
    run_tests()
