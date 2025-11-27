import xgboost as xgb
import sys
import json
import lief
import numpy as np
from scipy.stats import entropy as scipy_entropy
import time
import threading
from colorama import Fore, Style
import typer
from enum import Enum

app = typer.Typer()

class Colorblindness(str, Enum):
    protanopia = "protanopia"

def calculate_entropy(data):
    counts = np.bincount(np.frombuffer(data, dtype=np.uint8), minlength=256)
    probs = counts / len(data)
    probs = probs[probs > 0]
    return scipy_entropy(probs, base=2)

def extract_features_from_file(filepath: str):
    binary: lief.Binary = lief.parse(filepath)
    with open(filepath, "rb") as f:
        data = f.read()
    numstrings = [s for s in data if 32 <= s < 127] # rough estimation of chars
    hist, _ = np.histogram(np.frombuffer(data, dtype=np.uint8), bins=256, range=(0,256))
    return [
        len(data),
        binary.has_imports,
        binary.has_signatures,
        len(binary.sections) > 0,
        np.mean(hist),
        np.std(hist),
        np.max(hist),
        calculate_entropy(data),
        len(numstrings),
    ]
    print("Sorry, other platforms aren't supported yet. Only Windows is")
    sys.exit(1)

def checking_placeholder(filepath, done_predicting):
    while not done_predicting:
        for i in range(1,4):
            print(f"\rChecking if {filepath} is a malware{"."*i}", end="", flush=True)
            time.sleep(0.2)


@app.command()
def predict_malware(filepath: str = typer.Option(), model_path: str = typer.Option(), colorblindness: Colorblindness = typer.Option(None)):
    done_predicting = False
    thread = threading.Thread(target=checking_placeholder, args=(filepath, done_predicting), daemon=True)
    thread.start()

    model = xgb.XGBClassifier()
    model.load_model(model_path)

    features = extract_features_from_file(filepath)
    features = np.array(features).reshape(1, -1)
    pred = model.predict(features)

    done_predicting = True
    print()
    is_malware = True if pred[0] else False
    red = Fore.YELLOW if colorblindness == Colorblindness.protanopia else Fore.RED
    print(Style.BRIGHT + red + "[POSSIBLE THREAT]" if is_malware else Style.BRIGHT + Fore.GREEN + "[SAFE]", end=" ")
    print(Style.RESET_ALL + Fore.RESET, end="")
    print("It's a malware" if is_malware else "It's not a malware")

if __name__ == "__main__":
    app()
