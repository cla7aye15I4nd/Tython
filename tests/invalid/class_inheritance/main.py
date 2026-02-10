class Base:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

class Child(Base):
    pass
