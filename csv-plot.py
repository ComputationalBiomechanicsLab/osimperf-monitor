import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("results/RajagopalFreeFall.csv", names=['time', 'label', 'value'])

fig, ax = plt.subplots()

for label, group in df.groupby('label'):
    ax.plot(group['time'], group['value'], label=label)

ax.set_xlabel('Date')
ax.set_ylabel('WallTime')
ax.set_title('Plot from CSV Data')

ax.legend()

plt.show()
