class Pt:
    x: int
    y: int
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y
    def __str__(self) -> str:
        return "Pt"

def test_print_list_of_tuples() -> None:
    xs: list[tuple[int, int]] = [(1, 2), (3, 4)]
    print(xs)

def test_print_single_element_tuple() -> None:
    t: tuple[int] = (42,)
    print(t)

def test_print_numeric_tuple() -> None:
    t: tuple[int, float] = (1, 3.14)
    print(t)

def test_print_class_instance() -> None:
    p: Pt = Pt(10, 20)
    print(p)

def test_print_list_bytearray() -> None:
    xs: list[bytearray] = [bytearray(b"ab"), bytearray(b"cd")]
    print(xs)

def run_tests() -> None:
    test_print_list_of_tuples()
    test_print_single_element_tuple()
    test_print_numeric_tuple()
    test_print_class_instance()
    test_print_list_bytearray()
