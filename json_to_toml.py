import pytomlpp
import json
from pathlib import Path

def main():
    json_path = Path("config_json")
    toml_path = Path("config_toml")
    
    if not toml_path.exists() :
        toml_path.mkdir()
    
    for file in json_path.glob("*.json"):
        parts = file.stem.split("_")
        new_path = toml_path / f"scene-{parts[1]}-{parts[6]}.toml"
        with file.open("r") as json_file:
            data = json.load(json_file)
        
        with new_path.open("w") as toml_file:
            pytomlpp.dump(data, toml_file)
    
if __name__ == "__main__":
    main()
