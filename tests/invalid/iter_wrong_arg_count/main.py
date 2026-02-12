class C:
    def __iter__(self) -> "C":
        return self
    def __next__(self) -> int:
        return 1

it: C = iter(C(), C())
