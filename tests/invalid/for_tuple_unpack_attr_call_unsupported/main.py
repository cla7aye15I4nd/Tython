class Box:
    def make(self) -> list[int]:
        return [1, 2, 3]


def run_case() -> None:
    for a, b in Box().make():
        print(a, b)


if __name__ == "__main__":
    run_case()
