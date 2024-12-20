import re
import subprocess

import matplotlib.pyplot as plt

LTS_FILE_NAME = "python_benchmarks_tmp"


def plot ():
    start_states = 1000
    start_transitions = 1000
    step = 500
    end_states = 5000
    end_transitions = 5000

    pt_times = []
    for s in range(start_states, end_states + 1, step):
        for t in range(start_transitions, end_transitions + 1, step):
            pt = measure_time(s, t, True)
            pt_times.append((s, t, pt))
            print(f"finished {s}x{t}")

    print("(states, transitions, time in seconds)")
    print(f"pt: {pt_times}")

    x_coordinates = [x for (x, _, _) in pt_times]
    y_coordinates = [y for (_, y, _) in pt_times]
    times = [t for (_, _, t) in pt_times]

    fig = plt.figure()
    ax = fig.add_subplot(projection='3d')

    # Plot a basic wireframe.
    ax.scatter(x_coordinates, y_coordinates, times, marker='o')

    ax.set_xlabel("states")
    ax.set_ylabel("transitions")
    ax.set_zlabel("time [s]")

    plt.savefig("python_benchmark.pdf")
    plt.show()

def measure_time(states: int, transitions: int, pt: bool) -> float:
    subprocess.call(f"./target/release/ccs random-lts -s {states} -t {transitions} -a 1 >{LTS_FILE_NAME}", shell=True)

    if pt:
        flags = "-bp"
    else:
        flags = "-b"
    args = ["./target/release/ccs", "bisimilarity", flags, f"{LTS_FILE_NAME}"]
    result = subprocess.run(args, stdout=subprocess.PIPE)
    result = result.stdout.decode("utf-8")
    #todo maybe check if both bisimilarities are equal
    pattern = r"took (\d+\.?\d*\S*)"

    match = re.search(pattern, result)
    if match is None:
        raise ValueError("regex didnt match" + result)

    pt_time = parse_time(match.group(1))

    return pt_time

def parse_time (time:str) -> float:
    nano = re.match(r"(\d+\.?\d+)ns", time)
    if not nano is None:
        return float(nano.group(1)) * 1e-9

    micro = re.match(r"(\d+\.?\d+)µs", time)
    if not micro is None:
        return float(micro.group(1)) * 1e-6

    milli = re.match(r"(\d+\.?\d+)ms", time)
    if not milli is None:
        return float(milli.group(1)) * 1e-3

    seconds = re.match(r"(\d+\.?\d+)s", time)
    if not seconds is None:
        return float(seconds.group(1))

    raise ValueError("unexpected time format " + time)


def main():
    subprocess.call(["cargo", "build", "--release"])

    plot()

if __name__ == '__main__':
    main()
