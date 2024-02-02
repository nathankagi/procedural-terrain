# https://stackoverflow.com/questions/42147776/producing-2d-perlin-noise-with-numpy

import numpy as np
import matplotlib.pyplot as plt
import random

def perlin(x, y, seed=0):
    np.random.seed(seed)
    p = np.arange(256, dtype=int)
    np.random.shuffle(p)
    p = np.stack([p, p]).flatten()

    xi, yi = x.astype(int), y.astype(int)
    xf, yf = x - xi, y - yi
    u, v = fade(xf), fade(yf)

    n00 = gradient(p[p[xi] + yi], xf, yf)
    n01 = gradient(p[p[xi] + yi + 1], xf, yf - 1)
    n11 = gradient(p[p[xi + 1] + yi + 1], xf - 1, yf - 1)
    n10 = gradient(p[p[xi + 1] + yi], xf - 1, yf)
    
    x1 = lerp(n00, n10, u)
    x2 = lerp(n01, n11, u)
    return lerp(x1, x2, v)

def lerp(a, b, x):
    return a + x * (b - a)

def fade(t):
    return 6 * t**5 - 15 * t**4 + 10 * t**3

def gradient(h, x, y):
    vectors = np.array([[0, 1], [0, -1], [1, 0], [-1, 0]])
    g = vectors[h % 4]
    return g[:, :, 0] * x + g[:, :, 1] * y

r = 1000
p = np.zeros((r,r))
for i in range(4):
    freq = 2**i
    lin = np.linspace(0, freq, r, endpoint=False)
    x, y = np.meshgrid(lin, lin)
    p = perlin(x, y, seed=random.randint(0, 1000)) / freq + p

plt.imshow(p, origin='upper')
plt.show()