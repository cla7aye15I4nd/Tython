def takes_int(x: int) -> int:
    return x


def run_case() -> None:
    y: int = takes_int("a")


if __name__ == "__main__":
    run_case()
