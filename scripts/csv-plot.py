#!/usr/bin/python3

# This script will scan the results folder for any csv files and plot them.
import pandas as pd
import matplotlib.pyplot as plt

import os
import sys

file = sys.argv[1]
out = sys.argv[2]
print("file = ", file)
print("out = ", out)
file_name = os.path.splitext(file)[0]

df = pd.read_csv(file, names=['name', 'time', 'label', 'value'])

df['label']=pd.to_datetime(df['label'], format='%Y-%m-%d')

fig, ax = plt.subplots()

unique_labels = df['name'].unique()

for lab in unique_labels:
    filtered_data=df.loc[df['name'] == lab]
    ax.scatter(filtered_data['label'], filtered_data['value'], marker='o', label=lab)

ax.set_xlabel('Date')
ax.set_ylabel('WallTime')
ax.set_title(file_name)

plt.xticks(rotation=45)

plt.legend()
plt.tight_layout()
plt.savefig(out)
