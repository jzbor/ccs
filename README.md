# A Practical Implementation of the Caclulus of Communicating Systems

## Trying out the Program
With Cargo:
```console
$ cargo build --release
$ export PATH="$(realpath target/release/):$PATH"
```

With Nix:
```console
$ nix shell
```

## Running the Benchmark
There are two benchmarking scripts:
* `benchmark.py` to execute the benchmark
* `render_benchmark.py` to create diagram visualizations

Running the benchmarks with Nix:
```console
$ nix run .#benchmark
$ nix run .#render-benchmark
```

Running benchmarks without Nix (requires Cargo, Python 3 and matplotlib):
```console
$ cargo build --release
$ python3 benchmark.py
$ python3 render_benchmark.py
```

See `benchmark.py --help` for optional parameters or view the 3D diagram with `render_benchmark.py show`.

