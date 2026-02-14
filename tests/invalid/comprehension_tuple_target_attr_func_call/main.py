class Box:
    def pairs(self) -> list[tuple[int, int]]:
        return [(1, 2), (3, 4)]


def run_case() -> None:
    out: list[int] = [a + b for (a, b) in Box().pairs()]
    print(out)


if __name__ == "__main__":
    run_case()
