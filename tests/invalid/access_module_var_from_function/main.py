if __name__ == "__main__":
    x: int = 42

def foo() -> int:
    return x

if __name__ == "__main__":
    result: int = foo()
    print(result)
