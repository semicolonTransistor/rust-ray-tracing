import itertools
from pathlib import Path
import subprocess

TRIALS = 5

THREADS = (
    1, 4, 8, 12, 16, 20, 24
)

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

PREFIX = "Intel13700K-AVX-F32"

def main():
    test_cases = set()
    
    test_cases.update(itertools.product(
        (534,), 
        ((3840, 2160),),
        THREADS,
        ("scaler", "vectorized"),
        (100, )
        )
    )
    
    output_dir = Path("results")
    if not output_dir.exists():
        output_dir.mkdir()

    config_dir = Path("config_toml")
    
    executable_path = Path("target/release/ray-tracing")
    
    for i, test_case in enumerate(test_cases):
        for trial in range(TRIALS):
            print(f"Rendering scene {test_case[0]} at {test_case[1][0]}x{test_case[1][1]} using {test_case[2]} threads in {test_case[3]} mode at {test_case[4]} samples per pixel")
            print(f"trial {trial}")
            
            test_name = f"{PREFIX}-{test_case[0]}-{test_case[1][0]}x{test_case[1][1]}-{test_case[2]}-{test_case[3]}-{test_case[4]}-{trial}"
            
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
                
            print(f"Completed scene {test_case[0]} at {test_case[1][0]}x{test_case[1][1]} using {test_case[2]} threads in {test_case[3]} mode at {test_case[4]} samples per pixel")
        


if __name__ == "__main__":
    main()
