def main() -> None:
    try:
        raise ValueError("boom")
    except (ValueError, TypeError):
        x: int = 1
