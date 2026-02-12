class Counter:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

c: Counter = Counter(1)
c()
