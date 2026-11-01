#!/usr/bin/env python3
"""Generate selected model scene data from pinned client CSV exports."""

import argparse
import csv
import json
from pathlib import Path

SCENES = {596, 641, 727}


def read_rows(path, fields):
    with path.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source)
        if not reader.fieldnames or not set(fields).issubset(reader.fieldnames):
            raise ValueError(f"{path} is missing required columns {fields}")
        return list(reader)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-dir", type=Path, default=Path.home() / "Projects/wow/data")
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parents[1] / "crates/sim-data/data/model_scenes.rs")
    args = parser.parse_args()
    scene_rows = read_rows(args.input_dir / "UiModelScene.csv", ["ID", "UiSystemType"])
    actor_rows = read_rows(args.input_dir / "UiModelSceneActor.csv", ["ID", "UiModelSceneID", "ScriptTag"])
    camera_rows = read_rows(args.input_dir / "UiModelSceneCamera.csv", ["ID", "UiModelSceneID"])
    scenes = [(int(r["ID"]), int(r["UiSystemType"])) for r in scene_rows if int(r["ID"]) in SCENES]
    actors = [(int(r["ID"]), int(r["UiModelSceneID"]), r["ScriptTag"]) for r in actor_rows if int(r["UiModelSceneID"]) in SCENES]
    cameras = [(int(r["ID"]), int(r["UiModelSceneID"])) for r in camera_rows if int(r["UiModelSceneID"]) in SCENES]
    if {scene for scene, _ in scenes} != SCENES or {scene for _, scene, _ in actors} != SCENES or {scene for _, scene in cameras} != SCENES:
        raise ValueError("selected scenes, actors, or cameras are missing")
    for rows in (scenes, actors, cameras):
        if len({row[0] for row in rows}) != len(rows):
            raise ValueError("duplicate row ID")
    scene_values = ", ".join(f"({scene}, {kind})" for scene, kind in sorted(scenes))
    lines = ["//! Generated from UiModelScene{,Actor,Camera}.csv build 12.1.5.70391.", "", f"pub static SCENES: &[(i64, i64)] = &[{scene_values}];", "", "pub static ACTORS: &[(i64, i64, &str)] = &["]
    lines.extend(f"    ({actor}, {scene}, {json.dumps(tag, ensure_ascii=False)})," for actor, scene, tag in sorted(actors))
    camera_values = ", ".join(f"({camera}, {scene})" for camera, scene in sorted(cameras))
    lines.extend(["];", "", f"pub static CAMERAS: &[(i64, i64)] = &[{camera_values}];", ""])
    args.output.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {len(scenes)} scenes, {len(actors)} actors, {len(cameras)} cameras")


if __name__ == "__main__":
    main()
