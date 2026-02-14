class BadClassBodyExpr:
    1 + 2


def run_case() -> None:
    print("unreachable")


if __name__ == "__main__":
    run_case()
