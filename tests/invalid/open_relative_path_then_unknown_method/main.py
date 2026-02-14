def run_case() -> None:
    data: str = open("some_relative_file.txt")
    data.missing_method()


if __name__ == "__main__":
    run_case()
