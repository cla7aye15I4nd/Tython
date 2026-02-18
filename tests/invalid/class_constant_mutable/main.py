class Bad:
    VALUES: list[int] = [1, 2, 3]


BROKEN: int = Bad.VALUES


if __name__ == "__main__":
    print(Bad.VALUES)
