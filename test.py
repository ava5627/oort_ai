#!/usr/bin/env python

import argparse
import json
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="A simple command-line tool.")
    parser.add_argument("scenario_name", type=str, help="Name of the scenario to run.")
    parser.add_argument(
        "-n",
        "--no-commit",
        action="store_true",
        help="Run the scenario without committing changes.",
    )
    parser.add_argument(
        "-f",
        "--force",
        action="store_true",
        help="Force saving even if not a new best time.",
    )
    parser.add_argument(
        "--source",
        type=str,
        default="./target/bundle_output.rs",
        help="Path to the source file.",
    )
    args = parser.parse_args()
    scenario_name = args.scenario_name
    scenario_tutorial_name = f"tutorial_{scenario_name}"
    no_commit = args.no_commit

    oort_path = Path("/home/ava/repos/oort3/")
    ai_path = Path("/home/ava/repos/oort_ai/")
    source_path = ai_path / args.source
    scenario_path = oort_path / "shared/simulator/src/scenario/"
    scenario_ai_path = oort_path / "shared/builtin_ai/src/"

    if (scenario_path / f"{scenario_tutorial_name}.rs").exists():
        enemy = scenario_ai_path / f"tutorial/{scenario_tutorial_name}_enemy.rs"
        scenario_name = scenario_tutorial_name
    else:
        enemy = scenario_ai_path / "tutorial/tutorial_acceleration_initial.rs"

    if not enemy.exists():
        enemy = scenario_ai_path / "tutorial/tutorial_acceleration_initial.rs"

    print(f"Running scenario: {scenario_name}")
    print(f"Source file: {source_path}")
    if not source_path.exists():
        print("Source file does not exist!")
        return
    print(f"Enemy AI file: {enemy}")
    if not enemy.exists():
        print("Enemy AI file does not exist!")
        return
    cmd = [
        "/home/ava/repos/oort3/target/debug/battle",
        "-j",
        scenario_name,
        source_path.as_posix(),
        enemy.as_posix(),
    ]
    output = subprocess.check_output(
        " ".join(cmd), shell=True, text=True, env={"RUST_LOG": "error"}
    )
    result = json.loads(output)[0]
    if not Path("./times.json").exists():
        with open("./times.json", "w") as f:
            json.dump({}, f)
    with open("./times.json", "r") as f:
        best_times = json.load(f)
    s = "Times:  "
    for i, t in enumerate(result["times"]):
        best = best_times.get(scenario_name, {}).get("times", [])
        if i < len(best):
            if best[i] >= 10 and t < 10:
                s += f"{i}: {t:06.3f},  "
            else:
                s += f"{i}: {t:.3f},  "
        else:
            s += f"{i}: {t:.3f},  "
    s = s[:-3]
    print(s)
    if scenario_name in best_times:
        bests = "Bests:  "
        for i, best in enumerate(best_times.get(scenario_name, {}).get("times", [])):
            if result["times"][i] >= 10 and best < 10:
                bests += f"{i}: {best:06.3f},  "
            else:
                bests += f"{i}: {best:.3f},  "
        print(bests[:-3])
        diffs = "Diffs:    "
        for i, (best, current) in enumerate(zip(best_times[scenario_name]["times"], result["times"])):
            if current < best:
                diffs += "\033[32m-"
            elif current > best:
                diffs += "\033[31m+"
            else:
                diffs += "\033[34m "
            if result["times"][i] >= 10 and abs(current - best) < 10:
                diffs += f"{abs(current - best):06.3f}\033[0m,    "
            else:
                diffs += f"{abs(current - best):.3f}\033[0m,    "
        print(diffs[:-5])
        print(f"Best Time:    {best_times[scenario_name]['average_time']:.3f}", end=" ")
        if result["average_time"] < best_times[scenario_name]["average_time"]:
            print(
                f"\033[32m-{best_times[scenario_name]['average_time'] - result['average_time']:.3f}\033[0m"
            )
        elif result["average_time"] > best_times[scenario_name]["average_time"]:
            print(
                f"\033[31m+{result['average_time'] - best_times[scenario_name]['average_time']:.3f}\033[0m"
            )
        else:
            print()
    print(f"Average Time: {result['average_time']:.3f}")
    if len(result["losses"]) > 0 or len(result["draws"]) > 0:
        print(f"Losses: {result['losses']}")
        print(f"Draws: {result['draws']}")
        print(f"Wins : {result['wins']}")
    if (
        scenario_name not in best_times
        or result["average_time"] < best_times[scenario_name]["average_time"]
        or args.force
    ):
        print("New best time!")
        if not no_commit:
            best_times[scenario_name] = {
                "average_time": result["average_time"],
                "times": result["times"],
            }
            with open("./times.json", "w") as f:
                json.dump(best_times, f, indent=4)
            subprocess.run(["git", "add", "."], cwd=ai_path.as_posix())
            subprocess.run(
                [
                    "git",
                    "commit",
                    "-m",
                    f"New best {scenario_name}: {result['average_time']:.3f}",
                ],
                cwd=ai_path.as_posix(),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "tag", f"{args.scenario_name}-{result['average_time']:.3f}"],
                cwd=ai_path.as_posix(),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "push"],
                cwd=ai_path.as_posix(),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "push", "--tags"],
                cwd=ai_path.as_posix(),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )


if __name__ == "__main__":
    main()
