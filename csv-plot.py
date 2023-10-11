# This script will scan the results folder for any csv files and plot them.
import pandas as pd
import matplotlib.pyplot as plt

import os
import sys

file = sys.argv[1]
print("file = ", file)
file_name = os.path.splitext(file)[0]

df = pd.read_csv(file, names=['time', 'label', 'value'])

df['label']=pd.to_datetime(df['label'], format='%Y-%m-%d')

fig, ax = plt.subplots()
print(file_name)
ax.scatter(df['label'], df['value'], marker='o', label=file_name)

ax.set_xlabel('Date')
ax.set_ylabel('WallTime')
ax.set_title(file_name)

plt.xticks(rotation=45)

plt.tight_layout()
plt.show()
