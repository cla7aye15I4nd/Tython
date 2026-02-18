class Refl:
    DIFF: int = 0
    SPEC: int = 1
    LABEL: str = "spec"
    ENABLED: bool = True
    TAG: bytes = b"ok"


class Outer:
    class Inner:
        VALUE: int = 7


def test_class_constants() -> None:
    print('CHECK test_class_constant lhs:', Refl.DIFF)
    print('CHECK test_class_constant rhs:', 0)
    assert Refl.DIFF == 0

    print('CHECK test_class_constant lhs:', Refl.SPEC)
    print('CHECK test_class_constant rhs:', 1)
    assert Refl.SPEC == 1

    print('CHECK test_class_constant lhs:', Refl.LABEL)
    print('CHECK test_class_constant rhs:', "spec")
    assert Refl.LABEL == "spec"

    print('CHECK test_class_constant lhs:', Refl.ENABLED)
    print('CHECK test_class_constant rhs:', True)
    assert Refl.ENABLED == True

    print('CHECK test_class_constant lhs:', Refl.TAG)
    print('CHECK test_class_constant rhs:', b"ok")
    assert Refl.TAG == b"ok"

    print('CHECK test_class_constant lhs:', Outer.Inner.VALUE)
    print('CHECK test_class_constant rhs:', 7)
    assert Outer.Inner.VALUE == 7


def run_tests() -> None:
    test_class_constants()
