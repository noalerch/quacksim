#!/usr/bin/env python3
"""Plot wolf-sheep-grass metric files."""

import argparse
from pathlib import Path

import matplotlib.pyplot as pyplot
import pandas


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("metrics", nargs="+", type=Path, help="CSV metric files")
    parser.add_argument("--output", required=True, type=Path, help="output image path")
    return parser.parse_args()

# TODO: labels, axes etc. should be supplied per model. currently hard-coded wolf-sheep-grass
def main() -> None:
    arguments = parse_arguments()
    arguments.output.parent.mkdir(parents=True, exist_ok=True)

    figure, axes = pyplot.subplots(3, 1, sharex=True, figsize=(10, 8))
    for metrics_path in arguments.metrics:
        metrics = pandas.read_csv(metrics_path)
        label = metrics_path.stem
        axes[0].plot(metrics["tick"], metrics["sheep"], label=label)
        axes[1].plot(metrics["tick"], metrics["wolves"], label=label)
        axes[2].plot(metrics["tick"], metrics["grown_grass"], label=label)

    axes[0].set_ylabel("Sheep")
    axes[1].set_ylabel("Wolves")
    axes[2].set_ylabel("Grown grass")
    axes[2].set_xlabel("Tick")
    axes[0].legend()
    figure.tight_layout()
    figure.savefig(arguments.output)
    pyplot.close(figure)


if __name__ == "__main__":
    main()
