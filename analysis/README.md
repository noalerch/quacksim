# Analysis
Analysis e. g. plotting is based on a Python environment.
The analysis tools require Python 3.10 or newer.

## To run

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r analysis/requirements.txt
```

These commands create `.venv` in the repository root and install the pinned
packages from `requirements.txt`. 

## Commands
Generate a graph after recording a simulation run:

```sh
python analysis/plot.py runs/run-001.csv --output graphs/run-001.png
```
