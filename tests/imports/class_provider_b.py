class Vec2:
    x: int
    y: int
    z: int

    def __init__(self, x: int, y: int, z: int) -> None:
        self.x = x
        self.y = y
        self.z = z

    def sum(self) -> int:
        return self.x + self.y + self.z
