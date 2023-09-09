# This script will scan the results folder for any csv files and plot them.
import pandas as pd
import matplotlib.pyplot as plt

import os

csv_files=[f for f in os.listdir('results') if f.endswith('.csv')]
print("Available CSV files:")
for i, file in enumerate(csv_files, 1):
    print(f"{i}, {file}")

selected_file = int(input("Enter the number of the file you want to plot: ")) -1

selected_path = os.path.join('results', csv_files[selected_file])
selected_file_name = os.path.splitext(csv_files[selected_file])[0]

df = pd.read_csv(selected_path, names=['time', 'label', 'value'])

df['label']=pd.to_datetime(df['label'], format='%Y_%m_%d')

fig, ax = plt.subplots()
print(selected_file_name)
ax.scatter(df['label'], df['value'], marker='o', label=selected_file_name)

ax.set_xlabel('Date')
ax.set_ylabel('WallTime')
ax.set_title(selected_file_name)

plt.xticks(rotation=45)

plt.tight_layout()
plt.show()
