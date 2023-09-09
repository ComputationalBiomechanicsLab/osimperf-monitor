import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("results/RajagopalFreeFall.csv", names=['time', 'label', 'value'])

df['label']=pd.to_datetime(df['label'], format='%Y_%m_%d')

fig, ax = plt.subplots()

ax.plot(df['label'], df['value'])

ax.set_xlabel('Date')
ax.set_ylabel('WallTime')
ax.set_title('Plot from CSV Data')

plt.xticks(rotation=45)

plt.tight_layout()
plt.show()
