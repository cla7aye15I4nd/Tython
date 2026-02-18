pi: float = 3.141592653589793


e: float = 2.718281828459045


inf: float = 1.0e308 * 10.0


nan: float = 0.0 / 0.0


tau: float = 6.283185307179586


def sqrt(x: float) -> float:
    if x < 0.0:
        return 0.0 / 0.0
    if x == 0.0:
        return 0.0
    guess: float = x
    i: int = 0
    while i < 100:
        guess = (guess + x / guess) / 2.0
        i = i + 1
    return guess


def floor(x: float) -> int:
    n: int = int(x)
    if float(n) > x:
        return n - 1
    return n


def ceil(x: float) -> int:
    n: int = int(x)
    if float(n) < x:
        return n + 1
    return n


def fabs(x: float) -> float:
    if x < 0.0:
        return 0.0 - x
    return x


def fmod(x: float, y: float) -> float:
    return x % y


def exp(x: float) -> float:
    if x < 0.0:
        # Avoid catastrophic cancellation in the alternating Taylor series.
        return 1.0 / exp(0.0 - x)

    ln2: float = 0.6931471805599453
    k: int = int(x / ln2)
    r: float = x - float(k) * ln2

    # exp(r) via Taylor series around 0, where it converges quickly and stably.
    result: float = 1.0
    term: float = 1.0
    i: int = 1
    while i <= 40:
        term = term * r / float(i)
        result = result + term
        if fabs(term) < 1.0e-15:
            i = 41
        else:
            i = i + 1

    while k > 0:
        result = result * 2.0
        k = k - 1

    return result


def log(x: float) -> float:
    if x <= 0.0:
        return 0.0 / 0.0
    k: int = 0
    m: float = x
    while m > 2.0:
        m = m / 2.0
        k = k + 1
    while m < 0.5:
        m = m * 2.0
        k = k - 1
    t: float = (m - 1.0) / (m + 1.0)
    t2: float = t * t
    result: float = 0.0
    power: float = t
    i: int = 0
    while i < 40:
        n: int = 2 * i + 1
        result = result + power / float(n)
        power = power * t2
        i = i + 1
    result = 2.0 * result
    ln2: float = 0.6931471805599453
    return result + float(k) * ln2


def log2(x: float) -> float:
    ln2: float = 0.6931471805599453
    return log(x) / ln2


def log10(x: float) -> float:
    ln10: float = 2.302585092994046
    return log(x) / ln10


def sin(x: float) -> float:
    pi_val: float = 3.141592653589793
    two_pi: float = 6.283185307179586
    x = x % two_pi
    if x > pi_val:
        x = x - two_pi
    if x < 0.0 - pi_val:
        x = x + two_pi
    result: float = 0.0
    term: float = x
    x2: float = x * x
    i: int = 0
    while i < 20:
        result = result + term
        term = 0.0 - term * x2 / float((2 * i + 2) * (2 * i + 3))
        i = i + 1
    return result


def cos(x: float) -> float:
    pi_val: float = 3.141592653589793
    two_pi: float = 6.283185307179586
    x = x % two_pi
    if x > pi_val:
        x = x - two_pi
    if x < 0.0 - pi_val:
        x = x + two_pi
    result: float = 0.0
    term: float = 1.0
    x2: float = x * x
    i: int = 0
    while i < 20:
        result = result + term
        term = 0.0 - term * x2 / float((2 * i + 1) * (2 * i + 2))
        i = i + 1
    return result


def tan(x: float) -> float:
    return sin(x) / cos(x)


def pow(base: float, exponent: float) -> float:
    if exponent == 0.0:
        return 1.0
    if base == 0.0:
        return 0.0
    return exp(exponent * log(base))


def isnan(x: float) -> bool:
    return x != x


def isinf(x: float) -> bool:
    return x == inf or x == 0.0 - inf
