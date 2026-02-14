def inc(x: int) -> int:
    return x + 1


def run_case() -> None:
    t = (inc, 1)
    value: int = t[1]
    print(value)


if __name__ == "__main__":
    run_case()
