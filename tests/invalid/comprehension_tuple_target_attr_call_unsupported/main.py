class Box:
    def make(self) -> int:
        return 5


def run_case() -> None:
    out: list[int] = [x for x in Box().make()]
    print(out)


if __name__ == "__main__":
    run_case()
