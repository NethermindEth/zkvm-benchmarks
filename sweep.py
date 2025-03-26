import argparse
from itertools import product
import subprocess


def build_eval_command(program, prover, shard_size, filename, extra_arg=None):
    """Helper function to build the eval.sh command with consistent format"""
    # Determine the actual program name to pass to eval.sh
    actual_program = "reth" if program.startswith("reth") else program

    # Build the command
    cmd = [
        "bash",
        "eval.sh",
        actual_program,
        prover,
        str(shard_size),
        filename,
    ]

    # Add extra argument if provided
    if extra_arg is not None:
        cmd.append(str(extra_arg))

    return cmd


def run_benchmark(
    filename,
    trials,
    programs,
    provers,
    shard_sizes,
    blocks,
    fibonacci_inputs,
):
    option_combinations = product(programs, provers, shard_sizes)
    for program, prover, shard_size in option_combinations:
        if shard_size != shard_sizes[0] and prover != "sp1":
            # Skip shard size variations for provers other than SP1
            print(
                f"Skipping {program}/{prover} with shard size {shard_size} (only SP1 supports different shard sizes)"
            )
            continue

        print(f"Running: {program} {prover} {shard_size}")

        if program == "fibonacci":
            # Run fibonacci with each input value
            for fib_input in fibonacci_inputs:
                print(f"  With fibonacci input {fib_input}")
                for _ in range(trials):
                    cmd = build_eval_command(
                        program, prover, shard_size, filename, fib_input
                    )
                    subprocess.run(cmd)

        elif program.startswith("reth"):
            for block in blocks:
                print(f"  With block {block}")
                for _ in range(trials):
                    cmd = build_eval_command(
                        program, prover, shard_size, filename, block
                    )
                    subprocess.run(cmd)

        else:
            # Other programs without extra args
            for _ in range(trials):
                cmd = build_eval_command(program, prover, shard_size, filename)
                subprocess.run(cmd)


def main():
    parser = argparse.ArgumentParser(
        description="Run benchmarks with various combinations of options."
    )
    parser.add_argument(
        "--filename", default="benchmark", help="Filename for the benchmark"
    )
    parser.add_argument("--trials", type=int, default=1, help="Number of trials to run")
    parser.add_argument(
        "--programs",
        nargs="+",
        default=["loop", "fibonacci", "tendermint", "reth16", "reth30"],
        help="List of programs to benchmark",
        choices=["loop", "fibonacci", "tendermint", "reth", "reth1", "reth16", "reth30"],
    )
    parser.add_argument(
        "--provers",
        nargs="+",
        default=["sp1"],
        help="List of provers to use",
        choices=["sp1", "risc0", "lita", "jolt", "nexus", "zisk"],
    )
    parser.add_argument(
        "--shard-sizes",
        type=int,
        nargs="+",
        default=[21],
        help="List of shard sizes to use",
    )

    parser.add_argument(
        "--blocks",
        nargs="+",
        default=None,
        help="List of block numbers for reth tests",
    )

    parser.add_argument(
        "--fibonacci",
        nargs="+",
        type=int,
        default=[100, 1000, 10000, 300000],
        help="Input values for fibonacci benchmarks",
    )

    args = parser.parse_args()

    # Initialize blocks list
    blocks = []

    # Handle block arguments
    if args.blocks:
        # Start with the blocks provided via --blocks
        blocks = args.blocks

    # Append block numbers if respective flags are enabled
    if "reth1" in args.programs:
        blocks.append("22014900")
    if "reth16" in args.programs:
        blocks.append("17106222")
    if "reth30" in args.programs:
        blocks.append("19409768")

    # If no blocks were specified, use default block for backward compatibility
    if not blocks and any(p.startswith("reth") for p in args.programs):
        print(
            "Warning: No blocks specified. Using default block 17106222 for reth programs."
        )
        blocks = ["17106222"]

    run_benchmark(
        args.filename,
        args.trials,
        args.programs,
        args.provers,
        args.shard_sizes,
        blocks,
        args.fibonacci,
    )


if __name__ == "__main__":
    main()
