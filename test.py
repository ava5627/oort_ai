#!/usr/bin/env python

import argparse
import json
import subprocess
from pathlib import Path


def find_enemy_ai(scenario_name):
    oort_path = Path("~/repos/oort3/").expanduser()
    scenario_ai_path = oort_path / "shared/builtin_ai/src/"
    if scenario_name.startswith("tutorial_"):
        scenario_name = scenario_name[len("tutorial_") :]
    if (
        oort_path / f"shared/simulator/src/scenario/tutorial_{scenario_name}.rs"
    ).exists():
        enemy = scenario_ai_path / f"tutorial/tutorial_{scenario_name}_enemy.rs"
        scenario_name = f"tutorial_{scenario_name}"
    else:
        enemy = scenario_ai_path / "tutorial/tutorial_acceleration_initial.rs"

    if not enemy.exists():
        enemy = scenario_ai_path / "tutorial/tutorial_acceleration_initial.rs"

    return scenario_name, enemy


def run_oort(scenario_name, source_path, enemy_path):
    cmd = [
        "/home/ava/repos/oort3/target/debug/battle",
        "-j",
        scenario_name,
        source_path.as_posix(),
        enemy_path.as_posix(),
    ]
    output = subprocess.check_output(
        " ".join(cmd), shell=True, text=True, env={"RUST_LOG": "error", "RUST_BACKTRACE": "1"}
    )
    return json.loads(output)[0]


def read_best_times():
    if not Path("./times.json").exists():
        return {}
    with open("./times.json", "r") as f:
        return json.load(f)


def get_color(current, best):
    if current < best:
        return "\033[32m-"
    elif current > best:
        return "\033[31m+"
    else:
        return "\033[34m "


def print_result(result, best_times, scenario_name):
    time_s = "Times: "
    best_s = "Bests: "
    diff_s = "Diffs:   "
    if scenario_name not in best_times:
        best_times[scenario_name] = {
            "average_time": float("inf"),
            "times": [float("inf")],
        }

    times = result["times"]
    bests = best_times[scenario_name]["times"]
    for i, (t, b) in enumerate(zip(times, bests)):
        diff = abs(t - b)
        color = get_color(t, b)
        if t >= 10 or b >= 10:
            time_s += f"{i}: {t:06.3f}, "
            best_s += f"{i}: {b:06.3f}, "
            diff_s += f"{color}{diff:06.3f}\033[0m,   "
        else:
            time_s += f"{i}: {t:.3f}, "
            best_s += f"{i}: {b:.3f}, "
            diff_s += f"{color}{diff:.3f}\033[0m,   "
    print(time_s[:-2])
    print(best_s[:-2])
    print(diff_s[:-2])
    avg_time = result["average_time"]
    best_avg_time = best_times[scenario_name]["average_time"]
    diff = abs(avg_time - best_avg_time)
    color = get_color(avg_time, best_avg_time)
    print(f"Best Time: {best_avg_time:.3f} {color}{diff:.3f}\033[0m")
    print(f"Average Time: {avg_time:.3f}")
    if len(result["losses"]) > 0 or len(result["draws"]) > 0:
        print(f"Losses: {result['losses']}")
        print(f"Draws: {result['draws']}")
        print(f"Wins : {result['wins']}")


def save_result(scenario_name, result, best_times):
    best_times[scenario_name] = {
        "average_time": result["average_time"],
        "times": result["times"],
    }
    with open("./times.json", "w") as f:
        json.dump(best_times, f, indent=4)


def commit_changes(scenario_name, average_time):
    ai_path = Path(__file__).parent.resolve()
    subprocess.run(["git", "add", "."], cwd=ai_path.as_posix())
    subprocess.run(
        [
            "git",
            "commit",
            "-m",
            f"New best {scenario_name}: {average_time:.3f}",
        ],
        cwd=ai_path.as_posix(),
        stdout=subprocess.DEVNULL,
    )
    subprocess.run(
        ["git", "tag", f"{scenario_name}-{average_time:.3f}"],
        cwd=ai_path.as_posix(),
        stdout=subprocess.DEVNULL,
    )
    subprocess.run(
        ["git", "push", "--quiet"],
        cwd=ai_path.as_posix(),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    subprocess.run(
        ["git", "push", "--tags", "--quiet"],
        cwd=ai_path.as_posix(),
        stdout=subprocess.DEVNULL,
    )


def run_scenario(scenario_name, source_path, force, no_commit):
    scenario_name, enemy = find_enemy_ai(scenario_name)
    print(f"Running scenario: {scenario_name}")
    print(f"Source file: {source_path}")
    if not source_path.exists():
        print("Source file does not exist!")
        return
    print(f"Enemy AI file: {enemy}")
    if not enemy.exists():
        print("Enemy AI file does not exist!")
        return
    result = run_oort(scenario_name, source_path, enemy)
    best_times = read_best_times()
    print_result(result, best_times, scenario_name)
    new_best = (
        scenario_name not in best_times
        or result["average_time"] < best_times[scenario_name]["average_time"]
    )
    if new_best:
        print("New best time!")
    if (new_best or force) and not no_commit:
        print("Commit changes (y/n): ", end="")
        choice = input().lower().strip()
        if len(choice) == 0 or choice[0] != "y":
            print("Aborting commit.")
            return
        save_result(scenario_name, result, best_times)
        commit_changes(scenario_name, result["average_time"])


def run_all_scenarios(source_path):
    best_times = read_best_times()
    ai_path = Path(__file__).parent.resolve()
    source_path = ai_path / source_path
    print(f"Running all scenarios using {source_path}")
    for scenario_name in best_times.keys():
        scenario_name, enemy = find_enemy_ai(scenario_name)
        print(f"\r\033[K{scenario_name}: ", end="", flush=True)
        result = run_oort(scenario_name, source_path, enemy)
        avg_time = result["average_time"]
        best_avg_time = best_times[scenario_name]["average_time"]
        if len(result["losses"]) > 0 or len(result["draws"]) > 0:
            print(
                f"Losses/Draws {len(result['losses'])+len(result['draws'])}"
            )
        elif avg_time != best_avg_time:
            color = get_color(avg_time, best_avg_time)
            diff = abs(avg_time - best_avg_time)
            print(f"Old: {best_avg_time:.3f}, New: {avg_time:.3f} {color}{diff:.3f}\033[0m")
        print("\r\033[K", end="")


def main():
    parser = argparse.ArgumentParser(description="A simple command-line tool.")
    parser.add_argument("scenario_name", type=str, help="Name of the scenario to run.")
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "-n",
        "--no-commit",
        action="store_true",
        help="Run the scenario without committing changes.",
    )
    group.add_argument(
        "-f",
        "--force",
        action="store_true",
        help="Force saving even if not a new best time.",
    )
    parser.add_argument(
        "-s", "--source",
        type=str,
        default="./target/bundle_output.rs",
        help="Path to the source file.",
    )
    args = parser.parse_args()

    ai_path = Path(__file__).parent.resolve()
    source_path = ai_path / args.source

    if args.scenario_name == "all":
        run_all_scenarios(source_path)
    else:
        run_scenario(args.scenario_name, source_path, args.force, args.no_commit)


if __name__ == "__main__":
    main()
