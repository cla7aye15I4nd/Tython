def takes_bytes(x: bytes) -> int:
    return len(x)


def run_case() -> None:
    y: int = takes_bytes(1)


if __name__ == "__main__":
    run_case()
