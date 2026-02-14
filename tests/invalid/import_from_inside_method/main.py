class Bad:
    def __init__(self) -> None:
        return

    def f(self) -> None:
        from imports import module_a


def run_case() -> None:
    Bad()


if __name__ == "__main__":
    run_case()
