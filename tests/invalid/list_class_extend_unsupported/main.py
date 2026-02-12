class User:
    id: int

    def __init__(self, id: int) -> None:
        self.id = id


def main() -> None:
    users: list[User] = []
    others: list[User] = [User(1), User(2)]

    # Invalid: list[User].extend() is unsupported
    users.extend(others)
