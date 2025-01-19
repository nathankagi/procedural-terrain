import numpy as np

def new_point(a, b):
    return {"a": a, "b": b, "new": (a - b)}

def new_point_scaled(a, b):
    return {"a_scaled": a * 2, "b_scaled": b * 2, "new": a * 2 - (a - b)}



a = np.array([0, 5])
b = np.array([1, 5])

print(a[0]*2 - (a[0] - b[0]))
print(a[1]*2 - (a[1] - b[1]))

print(new_point(2*a, 2*b))
print(new_point_scaled(a, b))
