def clamp(x, minimum=None, maximum=None):
    x = min(x, maximum) if maximum is not None else x
    x = max(minimum, x) if minimum is not None else x
    return x
