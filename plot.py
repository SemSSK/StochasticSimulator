import pandas as pd
import matplotlib.pyplot as plt
file = "gillespie"
df = pd.read_csv(file +".csv")
df.plot(x=df.columns[-1], y=df.columns[:-1])
plt.savefig(file + ".svg")
