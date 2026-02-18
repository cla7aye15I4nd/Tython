def test_open_write_read_append_close() -> None:
    writer = open("file_ops_data.txt", "w")
    wrote1: int = writer.write("alpha")
    writer.close()

    appender = open("file_ops_data.txt", "a")
    wrote2: int = appender.write("beta")
    appender.close()

    reader = open("file_ops_data.txt", "r")
    data: str = reader.read()
    reader.close()

    default_reader = open("file_ops_data.txt")
    default_data: str = default_reader.read()
    default_reader.close()

    print("CHECK test_file_io lhs:", wrote1)
    print("CHECK test_file_io rhs:", 5)
    assert wrote1 == 5
    print("CHECK test_file_io lhs:", wrote2)
    print("CHECK test_file_io rhs:", 4)
    assert wrote2 == 4
    print("CHECK test_file_io lhs:", data)
    print("CHECK test_file_io rhs:", "alphabeta")
    assert data == "alphabeta"
    print("CHECK test_file_io lhs:", default_data)
    print("CHECK test_file_io rhs:", "alphabeta")
    assert default_data == "alphabeta"


def test_close_is_idempotent() -> None:
    f = open("file_ops_data.txt", "r")
    f.close()
    f.close()
    print("CHECK test_file_io lhs:", True)
    print("CHECK test_file_io rhs:", True)
    assert True


def run_tests() -> None:
    test_open_write_read_append_close()
    test_close_is_idempotent()
