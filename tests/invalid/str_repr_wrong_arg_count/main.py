def run_case() -> None:
    s: str = "hello"
    out: str = s.__repr__(42)


if __name__ == "__main__":
    run_case()
