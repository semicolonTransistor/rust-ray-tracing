import itertools
from pathlib import Path
import subprocess
from cpuinfo import get_cpu_info
import os

TRIALS = 5

RESOLUTIONS = (
    (1920, 1080),
    (3840, 2160)
)

SCENE_SIZES = (
    54,
    102,
    150,
    198,
    246,
    294,
    342,
    390,
    438,
    486,
    534,
)

DATA_WIDTH = (
    "F64",
    "F32"
)

# PREFIX = "AMD7800X3D-AVX-F64"

def main():
    test_cases = set()

    threads = (1,) + tuple(range(4, os.cpu_count(), 4))
    
    test_cases.update(itertools.product(
        (534,), 
        ((3840, 2160),),
        threads,
        ("scaler", "vectorized"),
        (100, )
        )
    )

    test_cases.update(itertools.product(
        (54, 102, 150, 198, 246, 294, 342, 342, 390, 438, 486, 534), 
        ((1920, 1080), (3840, 2160),),
        (os.cpu_count(),),
        ("vectorized", ),
        (100, )
        )
    )

    test_cases.update(itertools.product(
        (534,), 
        ((3840, 2160),),
        (os.cpu_count(),),
        ("vectorized",),
        (100, 200, 300, 400)
    ))

    for i, test_case in enumerate(test_cases):
        print(f"Test case {i}: {test_case[0]} at {test_case[1][0]}x{test_case[1][1]} using {test_case[2]} threads in {test_case[3]} mode at {test_case[4]} samples per pixel")
    
    output_dir = Path("results")
    if not output_dir.exists():
        output_dir.mkdir()

    config_dir = Path("config_toml")
    
    executable_path = Path("target/release/ray-tracing")

    cpu_name = "7800X3D"
    if "avx512f" in get_cpu_info()["flags"]:
        simd_extensions = ("AVX", "AVX512")
    else:
        simd_extensions = ("AVX",)
    
    compile_options = itertools.product(simd_extensions, DATA_WIDTH)

    for (simd_extension, data_width) in compile_options:
        prefix = f"{cpu_name}-{simd_extension}-{data_width}"

        feature_flags = tuple()
        if simd_extension == "AVX512":
            feature_flags += ("--features", "use_avx512")
        
        if data_width == "F32":
            feature_flags += ("--features", "single_precision")

        print(f"Compiling for {prefix} using feature flags {feature_flags}")

        # remove all compile outputs
        subprocess.run(
            (
                "cargo",
                "clean"
            )
        )

        p = subprocess.run(
            (
                "cargo", "build",
                "--release",
            ) + feature_flags,
            capture_output=True
        )

        with (output_dir / f"{prefix}-compile.stdout").open("wb") as f:
            f.write(p.stdout)
        
        with (output_dir / f"{prefix}-compile.stderr").open("wb") as f:
            f.write(p.stderr)
        
        p.check_returncode()

        print(f"Compilation for {prefix} completed")

        
        for i, test_case in enumerate(test_cases):
            for trial in range(TRIALS):
                print(f"{i}: Rendering scene {test_case[0]} at {test_case[1][0]}x{test_case[1][1]} using {test_case[2]} threads in {test_case[3]} mode at {test_case[4]} samples per pixel")
                print(f"trial {trial}")
                
                test_name = f"{prefix}-{test_case[0]}-{test_case[1][0]}x{test_case[1][1]}-{test_case[2]}-{test_case[3]}-{test_case[4]}-{trial}"
                
                scene_file = config_dir / f"scene-{test_case[0]}-objects.toml"
                camera_file = config_dir / f"scene-{test_case[0]}-camera.toml"
                
                stat_file = output_dir / f"{test_name}.toml"
                image_file = output_dir / f"{test_name}.png"
                
                std_out_file = output_dir / f"{test_name}.stdout"
                std_err_file = output_dir / f"{test_name}.stderr"
                
                p = subprocess.run(
                    (
                        executable_path,
                        "--scene", scene_file,
                        "--camera", camera_file,
                        "--report", stat_file,
                        "--width", str(test_case[1][0]),
                        "--height", str(test_case[1][1]),
                        "--output-image", image_file,
                        "--render-mode", str(test_case[3]),
                        "--samples-per-pixel", str(test_case[4]),
                        "--thread-count", str(test_case[2])
                    ),
                    capture_output=True 
                )
                
                with std_out_file.open("wb") as f:
                    f.write(p.stdout)
                
                with std_err_file.open("wb") as f:
                    f.write(p.stderr)
                    
                print(f"{i}: Completed scene {test_case[0]} at {test_case[1][0]}x{test_case[1][1]} using {test_case[2]} threads in {test_case[3]} mode at {test_case[4]} samples per pixel")
        


if __name__ == "__main__":
    main()
