def main() -> None:
    def inner() -> int:
        import imports.module_a
        return 1

    x: int = inner()
    print(x)
