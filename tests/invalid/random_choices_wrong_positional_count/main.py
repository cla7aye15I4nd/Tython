import random


def run_case() -> None:
    population: list[int] = [1, 2, 3]
    weights: list[float] = [0.2, 0.3, 0.5]
    random.choices(population, weights)


if __name__ == "__main__":
    run_case()
