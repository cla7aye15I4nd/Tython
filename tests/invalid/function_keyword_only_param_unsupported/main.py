def f(*, x: int) -> int:
    return x

if __name__ == "__main__":
    y: int = f(x=1)
