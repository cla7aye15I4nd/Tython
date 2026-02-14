import random


def run_case() -> None:
    xs: list[int] = [1, 2, 3]
    random.shuffle(xs, xs)


if __name__ == "__main__":
    run_case()
