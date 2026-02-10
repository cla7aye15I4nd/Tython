class Vec2:
    x: int
    y: int

    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

    def sum(self) -> int:
        return self.x + self.y



def make_vec2(x: int, y: int) -> Vec2:
    return Vec2(x, y)


def dot_vec2(a: Vec2, b: Vec2) -> int:
    return a.x * b.x + a.y * b.y
