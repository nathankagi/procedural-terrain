import csv
import numpy as np
import matplotlib.pyplot as plt

csv_file_path = 'output.csv'

data_array = np.loadtxt(csv_file_path, delimiter=',')

plt.imshow(data_array, origin='upper')
plt.show()