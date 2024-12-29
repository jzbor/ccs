import json
import os
import re
import subprocess
import sys

LTS_FILE_NAME = "/tmp/benchmark.ccs"
USE_PAIGE_TARJAN = False


def bench(binary: str, step_width: int, nsteps: int, use_pt: bool):
    pt_times = []
    for s in range(step_width, step_width * nsteps + 1, step_width):
        for t in range(step_width, step_width * nsteps + 1, step_width):
            pt = measure_time(binary, s, t, use_pt)
            pt_times.append((s, t, pt))
            print(f"finished {s}x{t}", file=sys.stderr)

    return pt_times

def measure_time(binary: str, states: int, transitions: int, pt: bool) -> float:
    subprocess.call(f"{binary} random-lts -s {states} -t {transitions} -a 1 >{LTS_FILE_NAME}", shell=True)

    if pt:
        algo = "paige-tarjan"
    else:
        algo = "naive"
    args = [binary, "bisimilarity", "-b", "-a", algo, LTS_FILE_NAME]
    result = subprocess.run(args, stdout=subprocess.PIPE)
    result = result.stdout.decode("utf-8")
    #todo maybe check if both bisimilarities are equal
    pattern = r"took (\d+\.?\d*\S*)"

    match = re.search(pattern, result)
    if match is None:
        raise ValueError("regex didnt match" + result)

    pt_time = parse_time(match.group(1))

    os.remove(LTS_FILE_NAME)

    return pt_time

def parse_time(time:str) -> float:
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


def usage():
    print("Usage:");
    print(f"  {sys.argv[0]} [binary] [step_width] [nsteps]");
    pass

def main():
    binary = "./target/release/ccs"
    step_width = 100000
    nsteps = 10
    outfile = "benchmark.json"

    if "-h" in sys.argv or "--help" in sys.argv or "help" in sys.argv:
        usage()
        return

    if len(sys.argv) > 1:
        binary = sys.argv[1]
    if len(sys.argv) > 2:
        step_width = int(sys.argv[2])
    if len(sys.argv) > 3:
        nsteps = int(sys.argv[3])

    print(f"Benchmarking {binary} with {nsteps} steps of width {step_width}")

    data = bench(binary, step_width, nsteps, USE_PAIGE_TARJAN)

    with open(outfile, "w") as write:
        json.dump(data, write)
        print(f"written data to {outfile}", file=sys.stderr)

if __name__ == '__main__':
    main()
