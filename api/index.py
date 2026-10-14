from flask import Flask, jsonify
from flask import request
import RNA
import random
import math
import sqlite3
import requests
import os
import pandas as pd
from flask import send_from_directory
from flask import abort
from flask_cors import CORS, cross_origin
from openpyxl import load_workbook
import subprocess
from pathlib import Path
import re

app = Flask(__name__)

API_DIR = Path(__file__).resolve().parent
AROS_DIR = API_DIR / "AROS"
MISC_DIR = AROS_DIR / "misc"

AROS_INPUT_FILE = MISC_DIR / "input_1.txt"
TEMP_DIR = MISC_DIR / "temp_folder"
TEMP_FILE = TEMP_DIR / "temp_file.txt"



@app.route("/mrna", methods=["GET"])
def mrna_suggestion():
    cutoff = 5
    query = request.args.get('query')

    sequence_result = ""
    suggestions = []

    base_url = "https://rest.uniprot.org/uniprotkb/search"
    response = requests.get(base_url +"?fields=accession%2Creviewed%2Cid%2Cprotein_name%2Cgene_names%2Corganism_name%2Csequence%2Cxref_embl&query=" + query + "&size=" + str(cutoff))

    if not response.ok:
        response.raise_for_status()

    try:
        data = response.json()
    except:
        print("Could not parse result")

    for entry in data['results']:
        name = entry['proteinDescription']['recommendedName']['fullName']['value']
        suggestions += [name]
    if (len(suggestions) == 0):
        suggestions = ["No match found"]


    if (len(suggestions) >= 1 and suggestions[0] == query):
            #TODO properly validate sequence source allow sequence selection
            for databaseRef in data['results'][0]['uniProtKBCrossReferences']:
                if (databaseRef['database'] == 'EMBL'):
                    for property in databaseRef['properties']:
                        if (property['key'] == 'MoleculeType' and property['value'] == 'mRNA'):
                            seq_id = databaseRef['id']


            seq = requests.get('https://www.ebi.ac.uk/ena/browser/api/fasta/M62654')
            #TODO maybe also return the first line with the metadata
            sequence_result = ''.join(seq.text.split('\n')[1:])
            suggestions = []
    
    

    return jsonify({"sequence": sequence_result, 
                "suggestions": suggestions})

@app.route("/suggest", methods=['GET'])
def mirna_suggestion():
    query = request.args.get('query')
    cutoff = 5
    suggestions = []
    sequence_result = ""
    with sqlite3.connect('./data/mir_database.db') as connection:
        cursor = connection.cursor()
        select_query = "SELECT * FROM MiRNAs WHERE name LIKE ?;"
        cursor.execute(select_query, ("%"+query+"%",))
        suggestion_results = cursor.fetchmany(cutoff)
        for suggestion in suggestion_results:
            suggestions += [suggestion[1]] 
        if(len(suggestions) >= 1 and (suggestions[0].split()[0] == ">" + query or suggestions[0] == query)):
            sequence_result = suggestion[2]
            suggestions = []
    
    return jsonify({"sequence": sequence_result, 
                "suggestions": suggestions})



@app.route("/construct", methods=["POST"])
@app.route("/api/python/construct", methods=["POST"])
def python_route():
    req_data = request.get_json()

    print("========== CONSTRUCT ROUTE REACHED ==========", flush=True)
    print(f"Request path: {request.path}", flush=True)
    print(
        f"miRNA1: {req_data['miRNA1']}\n"
        f"miRNA2: {req_data['miRNA2']}\n"
        f"protein: {req_data['protein']}\n",
        flush=True,
    )

    res_sequence, res_structure, original_structure, mfe = construct_design([
        req_data["miRNA1"],
        req_data["miRNA2"],
        req_data["protein"],
    ])

    return jsonify({
        "sequence": res_sequence,
        "generated_structure": res_structure,
        "original_structure": original_structure,
        "mfe": mfe,
    })

    

from pyfurnace import Origami

def comp(seq): # This is just generate the complementary domains -> since the script went trough a couple of iterations there are some redundancy in some of the functions, but I will clean that up later
    transcribe_dict = {
        'A': 'U',
        'U': 'A',
        'G': 'C',
        'C': 'G',
        'X': 'X'
    }
    rna_seq = ''
    for n in seq:
        if n in transcribe_dict:
            rna_seq += transcribe_dict[n]
        else:
            #rna_seq += n.swapcase()
            rna_seq += n
    return rna_seq[::-1]

def get_pair_map(structure, sequence):
    pair_map = {}
    stack = []
    comp_stack = []
    current_comp_start = ''
    for i, c in enumerate(structure):
        if c == '(':
            stack.append(i)
        elif c == ')':
            j = stack.pop()
            pair_map[i] = j
            pair_map[j] = i
        if sequence[i] == 'k' or sequence[i] == 'K':
            if current_comp_start == '':
                current_comp_start = sequence[i]
                comp_stack.append(i)
            elif current_comp_start == sequence[i]:
                comp_stack.append(i)
            else:
                j = comp_stack.pop()
                pair_map[i] = j
                pair_map[j] = i

    return pair_map

def get_named_domains(sequence):
    sequence = list(sequence)
    characters = ['A', 'U', 'G', 'C', 'N', 'K', 'k']
    in_region = False
    region_start = 0
    named_domains = {}
    for i, n in enumerate(sequence):
        if n in characters and not in_region:
            continue
        elif n in characters and in_region and sequence[i-1] in named_domains.keys():
            named_domains[sequence[i-1]].append([region_start, i-1]) #Append to existing
            sequence[i-1] = 'N'
            print(named_domains)
            in_region = False
        elif n in characters and in_region and not (sequence[i-1] in named_domains.keys()):
            named_domains[sequence[i-1]] = [[region_start, i-1]]
            print(named_domains)
            sequence[i-1] = 'N'
            in_region = False
        elif not (n in characters) and not in_region:
            in_region = True
            region_start = i
        else:
             sequence[i-1] = 'N'
    if in_region:
        if sequence[i] in named_domains.keys():
            named_domains[sequence[i]].append([region_start, i]) #Append to existing
        else:
            named_domains[sequence[i]] = [[region_start, i]]
    return named_domains

def mutate_seq(seq_in, structure, wobble_frequency=0.05):
    comp_dict = {'A': 'U', 'U': 'A', 'G': 'C', 'C': 'G'}
    nucleotides = ['A', 'U', 'G', 'C']
    named_domain_map = get_named_domains(seq_in)
    pair_map = get_pair_map(structure, seq_in)
    mut_seq = list(seq_in)

    for i in range(len(mut_seq)):
        if mut_seq[i] != 'N' and mut_seq[i] != 'K' and mut_seq[i] != 'k':
            continue  

        if i in pair_map:
            j = pair_map[i]
            if mut_seq[j] != 'N' and mut_seq[j] != 'K' and mut_seq[j] != 'k':
                mut_seq[i] = comp_dict[mut_seq[j]]
            else:       
                choice = random.choice(nucleotides)
                mut_seq[i] = choice
                if random.random() > (1 - wobble_frequency) and mut_seq[i] == 'G': # Adds wobble basepairs with the specified frequency
                    mut_seq[j] = 'U'

                else:
                    mut_seq[j] = comp_dict[choice]
        else:
            
            mut_seq[i] = random.choice(nucleotides)

    return ''.join(mut_seq), named_domain_map


def bp_distance_to_target(seq, target_structure):
    md = RNA.md()
    md.temperature = 37.0
    md.dangles = 1
    fc = RNA.fold_compound(seq, md)
    ss, mfe = fc.mfe()
    
    return RNA.bp_distance(ss, target_structure), ss, mfe



def output_file(
    domains_out,
    structure_out,
    path: Path = AROS_INPUT_FILE,
):
    if not isinstance(domains_out, str):
        domains_out = "".join(domains_out)

    if len(domains_out) != len(structure_out):
        raise ValueError(
            f"Sequence ({len(domains_out)}) and structure "
            f"({len(structure_out)}) must be the same length."
        )

    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)

    path.write_text(
        f"Sequence: {domains_out}\n"
        f"Structure: {structure_out}\n",
        encoding="utf-8",
    )

    return path



    
def input_file(path: Path, file_pattern: str = "temp_file_*.txt"):
    path = Path(path)
    files = sorted(path.glob(file_pattern))

    
    if not files and (path / "temp_file.txt").is_file():
        files = [path / "temp_file.txt"]

    if not files:
        raise FileNotFoundError(f"No output files matching {file_pattern} in {path}")

    candidates = [] 

    for full_path in files:
        sequence = structure = None
        ensemble_div = None

        with full_path.open("r", encoding="utf-8") as file:
            for line in file:
                line = line.strip()
                if line.startswith("Sequence:"):
                    sequence = line.split(":", 1)[1].strip()
                elif line.startswith("Structure:"):
                    structure = line.split(":", 1)[1].strip()
                elif line.startswith("Ensemble_div:"):
                    try:
                        ensemble_div = float(line.split(":", 1)[1].strip())
                    except ValueError:
                        ensemble_div = None

        
        if sequence is None or structure is None or len(sequence) != len(structure):
            print(f"Skipping malformed output file: {full_path}", flush=True)
            continue

        
        candidates.append((
            ensemble_div if ensemble_div is not None else float("inf"),
            sequence, structure, full_path.name,
        ))

    if not candidates:
        raise ValueError(f"No valid output files found in {path}")

    best = min(candidates, key=lambda c: c[0])
    print(f"Picked {best[3]} with Ensemble_div={best[0]} "
          f"out of {len(candidates)} candidates", flush=True)

    return best[1], best[2]

import shutil, tempfile

API_DIR = Path(__file__).resolve().parent
AROS_BIN = API_DIR / "bin" / "aros"

def run_aros(workdir: Path):
    exe = Path("/tmp/aros_bin")
    if not exe.exists():
        shutil.copy(AROS_BIN, exe)
        exe.chmod(0o755)
    env = {**os.environ, "RUST_BACKTRACE": "1"}
    result = subprocess.run([str(exe)], cwd=workdir, text=True,
                            capture_output=True, timeout=240, env=env)
    print(f"AROS exit code: {result.returncode}", flush=True)
    print(f"AROS stdout:\n{result.stdout}", flush=True)
    print(f"AROS stderr:\n{result.stderr}", flush=True)
    if result.returncode != 0:
        raise RuntimeError(f"AROS failed ({result.returncode}):\n{result.stderr}")
    return result.stdout


def hill_climb_design(seq_in, structure, max_steps=10000,
                       n_keep=5, temperature=1.0, cooling_rate=0.999, wobble_frequency=0.05): 
    comp_dict = {'A': 'U', 'U': 'A', 'G': 'C', 'C': 'G'}
    pair_map = get_pair_map(structure, seq_in)
    n_positions = [i for i, x in enumerate(seq_in) if not(x in comp_dict)]
    current, named_domains = mutate_seq(seq_in, structure, wobble_frequency=wobble_frequency)
    current_dist, current_ss, current_mfe = bp_distance_to_target(current, structure)
    print(named_domains)
    best_candidates = [(current_dist, current, current_ss, current_mfe)]
    if len(n_positions) == 0:
            return best_candidates
    print(seq_in + "\n")

    for step in range(max_steps):
        if best_candidates[0][0] == 0 and len(best_candidates) >= n_keep: # remove and len(best_candidates) >= n_keep to let the function iterate all 20 000 times
            break
        
        i = random.choice(n_positions)
        candidate = list(current)

        choice = random.choice(['A', 'U', 'G', 'C'])
        candidate[i] = choice
        if i in pair_map:
            j = pair_map[i]
            if random.random() > (1 - wobble_frequency) and choice == 'G': # Adds wobble basepairs with the specified frequency
                candidate[j] = 'U'
            else:
                candidate[j] = comp_dict[choice]
        #If in the named domains
        for name in named_domains.keys():
            for region in named_domains[name]:
                '''if i < region[0]:
                    break
                '''
                if region[0] <= i <= region[1]:
                    for reg in named_domains[name]:
                        candidate[reg[0] + i - region[0]] = choice
                    if name.swapcase() in named_domains.keys():
                        for reg in named_domains[name.swapcase()]:
                            candidate[reg[1] - i + region[0]] = comp_dict[choice]
                    break

        candidate = ''.join(candidate)
        dist, ss, mfe = bp_distance_to_target(candidate, structure)

        
        accept = dist <= current_dist  or random.random() < math.exp(-(dist - current_dist) / max(temperature, 1e-6))

        if accept:
            current = candidate
            current_dist = dist
            current_ss = ss
            current_mfe = mfe
            
            existing_seqs = {c[1] for c in best_candidates}
            if candidate not in existing_seqs:
                best_candidates.append((dist, candidate, ss, mfe))
                best_candidates.sort(key=lambda x: x[0])
                best_candidates = best_candidates[:n_keep]

        temperature *= cooling_rate
        
        if step % 1000 == 0:
            print(f"step {step}: current_dist={current_dist}, best_dist={best_candidates[0][0]}, temp={temperature:.4f}")
    return best_candidates


def multi_start_hill_climb(seq_in, structure,
                             n_runs=5, max_steps=15000, wobble_frequency=0.05): 
    results = []
    for run in range(n_runs):
        pool = hill_climb_design(seq_in, structure,
                                  max_steps=max_steps, n_keep=1, wobble_frequency = wobble_frequency)
        results.append(pool[0])
        print(f"Run {run}: best_dist={pool[0][0]}")

    results.sort(key=lambda x: x[3])

   
    return results

def mutate_n_bases(sequence: str) -> str:
    
    return "".join(
        random.choice("AUGC") if base == "N" else base
        for base in sequence
    )

def construct_design(domains):
    import random
    #RL3
    '''
    miRNA1 = "UAGCUUAUCAGACUGAUGUUGACU"
    miRNA2 = "UGGAGUGUGACAAUGGUGUUUG"
    '''
    miRNA1 = domains[0]
    miRNA2 = domains[1]
    RiboSwitch = "UUUUCCAUCAAGAACAGGCCACCAUGGAAAA"

    #Gen ="CGUAAAGGAGAAGAACUUUUCACUGGAGUUGUCCCAAUUCUUGUUGAAUUAGAUGGUGAUGUUAAUGGGCACAAAUUUUCUGUCAGUGGAGAGGGUGAAGGUGAUGCAACAUACGGAAAACUUACCCUUAAAUUUAUUUGCACUACUGGAAAACUACCUGUUCCAUGGCCAACACUUGUCACUACUUUCGGUUAUGGUGUUCAAUGCUUUGCGAGAUACCCAGAUCAUAUGAAACAGCAUGACUUUUUCAAGAGUGCCAUGCCCGAAGGUUAUGUACAGGAAAGAACUAUAUUUUUCAAAGAUGACGGGAACUACAAGACACGUGCUGAAGUCAAGUUUGAAGGUGAUACCCUUGUUAAUAGAAUCGAGUUAAAAGGUAUUGAUUUUAAAGAAGAUGGAAACAUUCUUGGACACAAAUUGGAAUACAACUAUAACUCACACAAUGUAUACAUCAUGGCAGACAAACAAAAGAAUGGAAUCAAAGUUAACUUCAAAAUUAGACACAACAUUGAAGAUGGAAGCGUUCAACUAGCAGACCAUUAUCAACAAAAUACUCCAAUUGGCGAUGGCCCUGUCCUUUUACCAGACAACCAUUACCUGUCCACACAAUCUGCCCUUUCGAAAGAUCCCAACGAAAAGAGAGAuCACAUGGUCCUUCUUGAGUUUGUAACAGCUGCUGGGAUUACACAUGGCAUGGAUGAACUAUACAAAUAA"

    Toehold = mutate_n_bases('N'*10)
    #Toehold = 'N'*10

    loop7 = "N" * 7

    #RL3
    RiboSwitchDisplace = mutate_n_bases('N'*13)
    #RiboSwitchDisplace = 'N'*13
    domain_order = (comp(miRNA1) + comp(miRNA2) + comp(RiboSwitchDisplace) + comp(Toehold) + miRNA2[7:]
                    + loop7 + miRNA1[7:] + Toehold + RiboSwitchDisplace + RiboSwitch + comp(RiboSwitchDisplace))

    structure_map = ('(' *len(comp(miRNA1)[7:]) + '.' *len(comp(miRNA1)[:7]) + '(' *len(comp(miRNA2)[7:]) + '.' *len(comp(miRNA2)[:7]) + '.' * len((comp(RiboSwitchDisplace) + comp(Toehold))) + ')' * len(miRNA2[7:]) + 
                     '.' * len(loop7) + ')' * len(miRNA1[7:]) + '.' * len(Toehold) + '(' * len(RiboSwitchDisplace) + '(' * 8 + '.' * (len(RiboSwitch) -16) + ')' * 8 +  ')' * len(comp(RiboSwitchDisplace)))



    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        (work / "misc" / "temp_folder").mkdir(parents=True)
        output_file(domain_order, structure_map, path=work / "misc" / "input_1.txt")
        run_aros(work)
        res_sequence, res_structure = input_file(path=work / "misc" / "temp_folder")

        print(res_sequence, flush=True)
        print(res_structure, flush=True)

        _, mfe = RNA.fold(res_sequence)
    
    print(structure_map)
    '''with open("./data/run20.txt", "w") as f:
                for res in results:
                     f.write(str(res))
                     f.write("\n")
                     f.write("Main strand: " +res[1])
                     f.write("\n")
                     f.write("\n")
    '''
    
    return [res_sequence,res_structure,structure_map,mfe,]


@app.route("/pdb", methods=['POST'])
def pdb_file():
    import pyfurnace as pf
    req_data = request.get_json()
    sequence = req_data['seq']
    structure = req_data['structure']
    print("Got " + sequence)
    print("Got " + structure)
    origami = Origami.from_structure(sequence=sequence, structure=structure)

    #Motif parser
    origami_motif_build = pf.Origami()
    current_line = [pf.Dovetail(0, up_cross=False)]
    for i, line in enumerate(origami):
        for j, motif in enumerate(line):
            #Stem
            if isinstance(motif, pf.Stem):
                current_line.append(motif)
            
            elif len(motif.sequence) != 0:
                #Other stuff
                #Loop "╮│╯"
                if len(origami._matrix[i][j + 1]) != 2:
                    clean_seq = "".join(c for c in origami._matrix[i][j][0].sequence if c in "ACGUNKMRSWYBDHVX&") 
                    loop = pf.Loop(open_left=True, sequence=clean_seq)
                    current_line.append(loop)
                #Single strand
                else:
                    current_line.append(pf.single_strand(sequence=origami._matrix[i][j].sequence))
        
        origami_motif_build.append(current_line)
        current_line.clear()
        current_line = [pf.Dovetail(0, down_cross=False)]

    
    origami = origami_motif_build
    path = "./data/pdb/" + str(random.random())[2:]
    #origami.save_3d_model(path, pdb=True)
    return jsonify({"path": path[2:]})

@app.route('/data/pdb/<path:filename>', methods=['GET'])
@cross_origin()
def serve_pdb(filename):
    PDB_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '.', 'data', 'pdb'))
    try:    
        return send_from_directory(PDB_DIR, filename, mimetype='chemical/x-pdb', as_attachment=True, download_name='structure.pdb')
    except FileNotFoundError:
        abort(404)


@app.route("/target", methods=['GET'])
def target_disease():
    query = request.args.get('query')
    df = pd.read_excel("./data/HMDD.xlsx")
    diease_assoc_rows = df.loc[df['disease'] == query]
    miRNAs = diease_assoc_rows['miRNA'].tolist()
    descriptions = diease_assoc_rows['description'].tolist()
    return jsonify({"results": miRNAs, "descriptions": descriptions})




def resolve_domains(sequence: str, names: str = "RK", rng=random) -> str:
    _COMP = {"A": "U", "U": "A", "G": "C", "C": "G"}
   
    seq = sequence
    for name in names:
        up, low = name.upper(), name.lower()
        up_runs = list(re.finditer(f"{up}+", seq))
        low_runs = list(re.finditer(f"{low}+", seq))
        runs = up_runs + low_runs
        if not runs:
            continue

        lengths = {m.end() - m.start() for m in runs}
        if len(lengths) != 1:
            raise ValueError(
                f"Domain '{name}' appears with different lengths: {sorted(lengths)}"
            )
        length = lengths.pop()

        domain = "".join(rng.choice("AUGC") for _ in range(length))
        rev_comp = "".join(_COMP[b] for b in reversed(domain))

        chars = list(seq)
        for m in up_runs:
            chars[m.start():m.end()] = domain
        for m in low_runs:
            chars[m.start():m.end()] = rev_comp
        seq = "".join(chars)

    return seq

@app.route("/advanced", methods=['POST'])
def advanced_construct():
    req_data = request.get_json() or {}
    sequence = req_data.get('sequence', '')
    structure = req_data.get('structure', '')

    if not sequence or len(sequence) != len(structure):
        return jsonify({"error": "sequence and structure must be non-empty and the same length"}), 400

    try:
        sequence = resolve_domains(sequence)
    except ValueError as e:
        return jsonify({"error": str(e)}), 400

    print(sequence, flush=True)
    print(structure, flush=True)

    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        (work / "misc" / "temp_folder").mkdir(parents=True)
        output_file(sequence, structure, path=work / "misc" / "input_1.txt")
        run_aros(work)
        try:
            res_sequence, res_structure = input_file(path=work / "misc" / "temp_folder")
        except FileNotFoundError:
            return jsonify({"error": "AROS could not design this input."}), 422

    _, mfe = RNA.fold(res_sequence)
    path = "./data/txt/" + str(random.random())[2:]
    return jsonify({
        "sequence": res_sequence,
        "generated_structure": res_structure,
        "mfe": mfe,
        "filepath": path,
    })


@app.route('/data/txt/<path:filename>', methods=['GET'])
@cross_origin()
def serve_txt(filename):
    TXT_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'data', 'txt'))
    try:    
        return send_from_directory(TXT_DIR, filename, mimetype='text/plain; charset=utf-8', as_attachment=True, download_name='result_summary.txt')
    except FileNotFoundError:
        abort(404)
