import math


class Random:
    _state: int

    def __init__(self, seed: int) -> None:
        self._state = seed % 2147483646 + 1

    def _next(self) -> int:
        self._state = self._state * 48271 % 2147483647
        return self._state

    def random(self) -> float:
        return float(self._next()) / 2147483647.0

    def randint(self, a: int, b: int) -> int:
        return a + int(self.random() * float(b - a + 1))

    def gauss(self, mu: float, sigma: float) -> float:
        s: float = 2.0
        u: float = 0.0
        v: float = 0.0
        while s >= 1.0:
            u = 2.0 * self.random() - 1.0
            v = 2.0 * self.random() - 1.0
            s = u * u + v * v
            if s == 0.0:
                s = 2.0
        fac: float = math.sqrt(-2.0 * math.log(s) / s)
        return mu + sigma * u * fac

    def shuffle(self, lst: list[str]) -> None:
        n: int = len(lst)
        i: int = n - 1
        while i > 0:
            j: int = self.randint(0, i)
            tmp: str = lst[i]
            lst[i] = lst[j]
            lst[j] = tmp
            i = i - 1

    def choices(self, population: list[int], weights: list[float]) -> list[int]:
        total: float = 0.0
        i: int = 0
        while i < len(weights):
            total = total + weights[i]
            i = i + 1
        r: float = self.random() * total
        cumulative: float = 0.0
        i = 0
        while i < len(weights):
            cumulative = cumulative + weights[i]
            if r <= cumulative:
                return [population[i]]
            i = i + 1
        return [population[len(population) - 1]]
