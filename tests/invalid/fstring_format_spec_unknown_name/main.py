def run_case() -> None:
    value: int = 12
    out: str = f"{value:{missing_width}d}"
    print(out)


if __name__ == "__main__":
    run_case()
