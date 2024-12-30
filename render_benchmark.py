import json
import os
import sys

import matplotlib.pyplot as plt

OUTDIR = "figures"

def unzip(data):
    x_coordinates = [x for (x, _, _) in data]
    y_coordinates = [y for (_, y, _) in data]
    times = [t for (_, _, t) in data]
    return (x_coordinates, y_coordinates, times)


def render_3d(data, show=False):
    fig = plt.figure()
    ax = fig.add_subplot(projection='3d')

    x_coordinates, y_coordinates, times = unzip(data)
    ax.scatter(x_coordinates, y_coordinates, times, marker = 'o')

    ax.set_xlabel("number of states")
    ax.set_ylabel("number of transitions")
    ax.set_zlabel("time in seconds")
    ax.set_zlim(bottom=0)

    plt.savefig(f"{OUTDIR}/bench3d.svg");
    plt.savefig(f"{OUTDIR}/bench3d.png");

    if show:
        plt.show()

def render_states(filename: str, transitions: int, data):
    fig = plt.figure()
    ax = fig.add_subplot()

    low_data = [(st, tr, ti) for (st, tr, ti) in data if tr == transitions]

    x_coordinates, _, times = unzip(low_data)
    ax.plot(x_coordinates, times, marker = 'o')

    ax.set_xlabel(f"number of states ({transitions} transitions)")
    ax.set_ylabel("time in seconds")
    ax.set_ylim(bottom=0)

    plt.savefig(f"{OUTDIR}/{filename}.svg");
    plt.savefig(f"{OUTDIR}/{filename}.png");

def render_transitions(filename: str, states: int, data):
    fig = plt.figure()
    ax = fig.add_subplot()

    low_data = [(st, tr, ti) for (st, tr, ti) in data if st == states]

    _, x_coordinates, times = unzip(low_data)
    ax.plot(x_coordinates, times, marker = 'o')

    ax.set_xlabel(f"number of transitions ({states} states)")
    ax.set_ylabel("time in seconds")
    ax.set_ylim(bottom=0)

    plt.savefig(f"{OUTDIR}/{filename}.svg");
    plt.savefig(f"{OUTDIR}/{filename}.png");

def render_ratio(filename: str, rstates: int, rtransitions: int, data):
    fig = plt.figure()
    ax = fig.add_subplot()

    low_data = [(st, tr, ti) for (st, tr, ti) in data if rstates * st == rtransitions * tr]

    x_coordinates, _, times = unzip(low_data)
    ax.plot(x_coordinates, times, marker = 'o')

    ax.set_xlabel(f"number of states ({rstates}:{rtransitions} states to transitions)")
    ax.set_ylabel("time in seconds")
    ax.set_ylim(bottom=0)

    plt.savefig(f"{OUTDIR}/{filename}.svg");
    plt.savefig(f"{OUTDIR}/{filename}.png");

def main():
    infile = "benchmark.json"

    with open(infile, "r") as read:
        data = json.load(read)
        print(f"read data from {infile}", file=sys.stderr)

    print(data)

    if not os.path.exists(OUTDIR):
        os.makedirs(OUTDIR)

    render_3d(data, "show" in sys.argv )
    if "show" in sys.argv:
        return
    render_states("states_low", data[0][1], data)
    render_states("states_high", data[len(data) - 1][1], data)
    render_states("states_med", data[len(data) -1][1] // 2, data)
    render_transitions("transitions_low", data[0][0], data)
    render_transitions("transitions_high", data[len(data) - 1][0], data)
    render_transitions("transitions_med", data[len(data) -1][0] // 2, data)
    render_ratio("1to1", 1, 1, data)
    render_ratio("2to1", 2, 1, data)
    render_ratio("1to2", 1, 2, data)


if __name__ == '__main__':
    main()

