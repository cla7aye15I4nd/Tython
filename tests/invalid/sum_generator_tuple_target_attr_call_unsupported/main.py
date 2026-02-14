class Box:
    def make(self) -> int:
        return 5


def run_case() -> None:
    total: int = sum((x for x in Box().make()), 0)
    print(total)


if __name__ == "__main__":
    run_case()
