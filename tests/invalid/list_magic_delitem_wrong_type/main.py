def run_case() -> None:
    xs: list[int] = [1, 2]
    xs.__delitem__("0")


if __name__ == "__main__":
    run_case()
