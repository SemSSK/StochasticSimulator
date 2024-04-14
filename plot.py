import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("./gillespie.csv")
df.plot(x=df.columns[-1], y=df.columns[:-1])
plt.savefig("plot.svg")
