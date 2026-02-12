class User:
    id: int

    def __init__(self, id: int) -> None:
        self.id = id


class Admin:
    id: int

    def __init__(self, id: int) -> None:
        self.id = id


def main() -> None:
    users: list[User] = []
    users.append(User(1))

    # Invalid: list[User] cannot append Admin
    users.append(Admin(2))
