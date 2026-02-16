class Factory:
    pass


def run_case() -> None:
    f: Factory = Factory()
    f(1)


if __name__ == "__main__":
    run_case()
