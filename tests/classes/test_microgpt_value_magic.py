class Value:
    data: float

    def __init__(self, data: float) -> None:
        self.data = data

    def __add__(self, other: 'Value') -> 'Value':
        return Value(self.data + other.data)

    def __mul__(self, other: 'Value') -> 'Value':
        return Value(self.data * other.data)

    def __pow__(self, other: 'Value') -> 'Value':
        return Value(self.data ** other.data)

    def relu(self) -> 'Value':
        if self.data > 0.0:
            return Value(self.data)
        return Value(0.0)

    def __neg__(self) -> 'Value':
        return self * Value(-1.0)

    def __radd__(self, other: 'Value') -> 'Value':
        return self + other

    def __sub__(self, other: 'Value') -> 'Value':
        return self + (-other)

    def __rsub__(self, other: 'Value') -> 'Value':
        return other + (-self)

    def __rmul__(self, other: 'Value') -> 'Value':
        return self * other

    def __truediv__(self, other: 'Value') -> 'Value':
        return self * (other ** Value(-1.0))

    def __rtruediv__(self, other: 'Value') -> 'Value':
        return other * (self ** Value(-1.0))


def test_value_magic_arithmetic() -> None:
    a: Value = Value(2.0)
    b: Value = Value(3.0)

    add_v: Value = a + b
    mul_v: Value = a * b
    pow_v: Value = a ** b
    neg_v: Value = -a
    sub_v: Value = a - b
    div_v: Value = b / a

    print('CHECK test_microgpt_value_magic lhs:', add_v.data)
    print('CHECK test_microgpt_value_magic rhs:', 5.0)
    assert add_v.data == 5.0

    print('CHECK test_microgpt_value_magic lhs:', mul_v.data)
    print('CHECK test_microgpt_value_magic rhs:', 6.0)
    assert mul_v.data == 6.0

    print('CHECK test_microgpt_value_magic lhs:', pow_v.data)
    print('CHECK test_microgpt_value_magic rhs:', 8.0)
    assert pow_v.data == 8.0

    print('CHECK test_microgpt_value_magic lhs:', neg_v.data)
    print('CHECK test_microgpt_value_magic rhs:', -2.0)
    assert neg_v.data == -2.0

    print('CHECK test_microgpt_value_magic lhs:', sub_v.data)
    print('CHECK test_microgpt_value_magic rhs:', -1.0)
    assert sub_v.data == -1.0

    print('CHECK test_microgpt_value_magic lhs:', div_v.data)
    print('CHECK test_microgpt_value_magic rhs:', 1.5)
    assert div_v.data == 1.5


def test_value_magic_complex_chain() -> None:
    xs: list[Value] = [
        Value(1.0),
        Value(2.0),
        Value(3.0),
        Value(4.0),
    ]

    acc: Value = Value(0.0)
    for x in xs:
        acc = acc + x
        acc = acc + (Value(2.0) * x)
        acc = acc - (x / Value(2.0))

    acc = acc.relu()
    score: Value = (acc ** Value(2.0)) / Value(100.0)

    print('CHECK test_microgpt_value_magic lhs:', acc.data)
    print('CHECK test_microgpt_value_magic rhs:', 25.0)
    assert acc.data == 25.0

    print('CHECK test_microgpt_value_magic lhs:', score.data)
    print('CHECK test_microgpt_value_magic rhs:', 6.25)
    assert score.data == 6.25


def run_tests() -> None:
    test_value_magic_arithmetic()
    test_value_magic_complex_chain()
