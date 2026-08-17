from flask import Flask, jsonify
from flask import request, send_file
import pandas as pd
import RNA
import random
import math
import sqlite3
import requests
import io, os, shutil, tempfile
from flask import send_from_directory
from flask import abort
from flask_cors import CORS, cross_origin

app = Flask(__name__)




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
    with sqlite3.connect('../data/mir_database.db') as connection:
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



@app.route("/construct", methods=['POST'])
def python_route():
    if request.method == 'POST':
        req_data = request.get_json()
        #TODO Validate incoming data
        #TODO Change defualt, just for testing
        print(f"miRNA1: {req_data['miRNA1']} \n miRNA2: {req_data['miRNA2']} \n spacer: {req_data['spacer']} \n rbs_site: {req_data['rbs_site']} \n protein: {req_data['protein']} \n toehold: {req_data['toehold']} \n")
        [res_sequence, res_structure, disp_structure, mfe] = construct_design([req_data['miRNA1'], req_data['miRNA2'], req_data['spacer'], req_data['rbs_site'], req_data['protein'], req_data['toehold']])

        return jsonify({"sequence": res_sequence, 
                "original_exp_structure": res_structure,
                "disp_structure": disp_structure,
                "mfe": mfe})
                #"pdb_content_bytes": pdb_content})
    else: 
        print("Type of request is not POST")
    


from pyfurnace.generate.road import generate_road
from pyfurnace import Origami

def comp(seq): # This is just generate the complementary domains -> since the script went trough a couple of iterations there are some redundancy in some of the functions, but I will clean that up later
    transcribe_dict = {
        'A': 'U',
        'U': 'A',
        'G': 'C',
        'C': 'G',
        'X': 'X',
        'K' : 'k'
    }
    rna_seq = ''
    for n in seq:
        if n in transcribe_dict:
            rna_seq += transcribe_dict[n]
        else:
            rna_seq += n.swapcase()
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



def mutate_seq(seq_in, structure, wobble_frequency=0.05):
    comp_dict = {'A': 'U', 'U': 'A', 'G': 'C', 'C': 'G'}
    nucleotides = ['A', 'U', 'G', 'C']
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

    return ''.join(mut_seq)


def bp_distance_to_target(seq, target_structure):
    md = RNA.md()
    md.temperature = 37.0
    md.dangles = 1
    fc = RNA.fold_compound(seq, md)
    ss, mfe = fc.mfe()
    
    return RNA.bp_distance(ss, target_structure), ss, mfe



def hill_climb_design(seq_in, structure, pair_map, n_positions, max_steps=10000,
                       n_keep=5, temperature=1.0, cooling_rate=0.999, wobble_frequency=0.05): 
    comp_dict = {'A': 'U', 'U': 'A', 'G': 'C', 'C': 'G'}

    current = mutate_seq(seq_in, structure, wobble_frequency=wobble_frequency)  
    current_dist, current_ss, current_mfe = bp_distance_to_target(current, structure)
    
    best_candidates = [(current_dist, current, current_ss, current_mfe)]

    print(seq_in + "\n")
    print(n_positions)

    for step in range(max_steps):
        if best_candidates[0][0] == 0 and len(best_candidates) >= n_keep: # remove and len(best_candidates) >= n_keep to let the function iterate all 20 000 times
            break
        
        i = random.choice(n_positions)
        candidate = list(current)

        if i in pair_map:
            j = pair_map[i]
            choice = random.choice(['A', 'U', 'G', 'C'])
            candidate[i] = choice
            if random.random() > (1 - wobble_frequency) and choice == 'G': # Adds wobble basepairs with the specified frequency
                candidate[j] = 'U'
            else:
                candidate[j] = comp_dict[choice]
        else:
            candidate[i] = random.choice(['A', 'U', 'G', 'C'])

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


def multi_start_hill_climb(seq_in, structure, pair_map, n_positions,
                             n_runs=1, max_steps=500, wobble_frequency=0.05): 
    results = []
    for run in range(n_runs):
        pool = hill_climb_design(seq_in, structure, pair_map, n_positions,
                                  max_steps=max_steps, n_keep=1, wobble_frequency = wobble_frequency)
        results.append(pool[0])
        print(f"Run {run}: best_dist={pool[0][0]}")

    results.sort(key=lambda x: x[3])

   
    return results



def construct_design(domains):
    ''' miRNA_1 = domains[0] 
    miRNA_2 = domains[1]
    miRNA1_th = comp(miRNA_1[:1]) 
    # miRNA2_th = comp(miRNA_2[:1]) 
    spacer_1 = domains[2] 
    spacer_2 = spacer_1[:2] 
    spacer_3 = spacer_1[:1] 
    rbs_site = domains[3]
    miRNA_1 = miRNA_1[2:]
    miRNA_2 = miRNA_2[2:]
    #stem = domains[4]
    #mrna = domains[5]
    #toehold = domains[6]
    stem = 'NNNNNNNN'
    mrna = domains[4]
    toehold = domains[5]
    poly_c_domain = 'C'*3
    loop = 'N'*6
    poly_g_domain = 'G'*3
    clamp = 'AGC'
    poly_a_domain = 'A'*len(spacer_2)
    poly_u_domain = 'U'*3
    middle_stem = 'N'*(len(miRNA_1) + len(miRNA_2))
    algorithm_selector = 'Hill'
    print("0")
    
    #Standard
    domain_order = (
        miRNA_1 + poly_c_domain + miRNA_2 + loop + comp(toehold) + comp(miRNA_2)
        + poly_u_domain + comp(miRNA_1) + 'N'*20 + middle_stem + spacer_2 + miRNA1_th
        + comp(miRNA_1) + spacer_3 + miRNA1_th + comp(miRNA_2) + spacer_2 + middle_stem
        +'N'*10 + comp(miRNA_2) + poly_g_domain + comp(miRNA_1) + comp(clamp) + spacer_1
        + stem + rbs_site + comp(stem) + 'AUG' + spacer_3 + clamp + miRNA_1 + poly_u_domain +   # <-- changed `+ stem +` to `+ comp(stem) +`
        'N'*3 + comp(miRNA_1[:6]) + comp(clamp) + mrna
        ).upper()

    structure_map = (
    len(miRNA_1)*'(' + len(poly_c_domain)*'.' + len(miRNA_2)*'(' + len(loop)*'.' + len(toehold)*'.' + len(miRNA_2)*')'
        + len(poly_u_domain)*'.' + len(miRNA_1)*')' + '.'*20 + len(middle_stem)*'(' + len(spacer_2)*'.' + len(miRNA1_th)*'.'
        + len(miRNA_1)*'.' + len(spacer_3)*'.' + len(miRNA1_th)*'.' + len(miRNA_2)*'.' + len(spacer_2)*'.' + len(middle_stem)*')'
        +'.'*10 + len(miRNA_2)*'.' + len(poly_g_domain)*'(' + len(miRNA_1)*'(' + len(clamp)*'(' + len(spacer_1)*'.'
        + len(stem)*'(' + len(rbs_site)*'.' + len(stem)*')' + '.'*3 + len(spacer_3)*'.' + len(clamp)*')' + len(miRNA_1)*')' + len(poly_u_domain)*')' +
        '.'*3 + len(miRNA_1[:6])*'.' + len(clamp)*'.' + len(mrna)*'.'
    )
    '''
    #Alternative

    #Cancer miRNAs
    miRNA1 = "GCUCUGACGAGGUUGCACUACU"

    miRNA2 = "ACUCUUUCCCUGUUGCACUAC"

    #HIV miRNAs
    '''miRNA1 = "UAGGUAGUUUCCUGUUGUUGGG"
    
    miRNA2 = "UGGAUUUUUGGAUCAGGGA"
    '''


    miRNA3 = "UUUGGCAAUGGUAGAACUCACACU"

    miRNA4 = "UGUGCAAAUCCAUGCAAAACUGA"
    miRNA4 = ""

    RiboSwitch = "CUCGUUUAUCCCUGCAUACAGAAACAGAGGAGAUAUGCAAUGAUAAACGAGAACCUGGCG"

    #Gen ="CGUAAAGGAGAAGAACUUUUCACUGGAGUUGUCCCAAUUCUUGUUGAAUUAGAUGGUGAUGUUAAUGGGCACAAAUUUUCUGUCAGUGGAGAGGGUGAAGGUGAUGCAACAUACGGAAAACUUACCCUUAAAUUUAUUUGCACUACUGGAAAACUACCUGUUCCAUGGCCAACACUUGUCACUACUUUCGGUUAUGGUGUUCAAUGCUUUGCGAGAUACCCAGAUCAUAUGAAACAGCAUGACUUUUUCAAGAGUGCCAUGCCCGAAGGUUAUGUACAGGAAAGAACUAUAUUUUUCAAAGAUGACGGGAACUACAAGACACGUGCUGAAGUCAAGUUUGAAGGUGAUACCCUUGUUAAUAGAAUCGAGUUAAAAGGUAUUGAUUUUAAAGAAGAUGGAAACAUUCUUGGACACAAAUUGGAAUACAACUAUAACUCACACAAUGUAUACAUCAUGGCAGACAAACAAAAGAAUGGAAUCAAAGUUAACUUCAAAAUUAGACACAACAUUGAAGAUGGAAGCGUUCAACUAGCAGACCAUUAUCAACAAAAUACUCCAAUUGGCGAUGGCCCUGUCCUUUUACCAGACAACCAUUACCUGUCCACACAAUCUGCCCUUUCGAAAGAUCCCAACGAAAAGAGAGAuCACAUGGUCCUUCUUGAGUUUGUAACAGCUGCUGGGAUUACACAUGGCAUGGAUGAACUAUACAAAUAA"

    Toehold = 'K' *10

    #Toehold2 = 'K' * 10

    loop7 = "N" * 7

    algorithm_selector = 'Hill'

    '''
    #Linnea 1
    domain_order = (miRNA1[7:] + comp(RiboSwitch[:15]) + comp(Toehold2) + RiboSwitch[7:15] + comp(miRNA1[7:]) + Toehold 
                    + comp(miRNA2[7:]) + comp(miRNA2[:7]) + comp(miRNA1[7:]) + comp(miRNA1[:7]) + Toehold2 + RiboSwitch 
                    )#+ Gen)

    structure_map = ( '(' * len(miRNA1[7:] + comp(RiboSwitch[7:15])) + '.' * len( RiboSwitch[:7] + comp(Toehold2)) + 
                     ')' * len(RiboSwitch[7:15] + comp(miRNA1[7:])) + '.' * len(Toehold 
                    + comp(miRNA2[7:]) + comp(miRNA2[:7]) + comp(miRNA1[7:]) + comp(miRNA1[:7]) + Toehold2)
                    + '(' * 9 + '.' * 3 + '(' * 6 + '.' * 15 + ')' * 6 + '.' * 3 + ')' * 9 + '.' * 9 #RiboSwitch internal pairings
                    )# +'.' *len(Gen)) 
        
    '''
    #Linnea 2
    domain_order = (comp(miRNA1) + comp(miRNA2) + comp(miRNA3) + comp(miRNA4) + Toehold + RiboSwitch) #Without key strand

    structure_map = ('(' * len(miRNA2[7:]) + '.' * 7 + '(' * len(miRNA1[7:]) + '.' * len(comp(Toehold) + comp(RiboSwitch[:15])) +  #Key strand structure
                     ')' * len(comp(miRNA1[7:])) + '.' * len(comp(miRNA1[:7])) + ')' * len(comp(miRNA2[7:]))  + '.' * len(comp(miRNA2[:7]))) #Main strand base

    #TODO Generalize
    #Adding the variable parts depending on the number of input miRNAs
    key_strand = (miRNA2[7:] + loop7 + miRNA1[7:] + comp(RiboSwitch[:15]) + comp(Toehold))
    if miRNA3 != "":
        key_strand = (miRNA3[7:] + loop7) + key_strand
        structure_map = ('(' * len(miRNA3[7:]) + '.' * 7) + structure_map
        structure_map +=  ')' * len(comp(miRNA3[7:])) + '.' * len(comp(miRNA3[:7]))

    if miRNA4 != "":
            key_strand = (miRNA4[7:] + loop7) + key_strand
            structure_map = ('(' * len(miRNA4[7:]) + '.' * 7) + structure_map
            structure_map +=  ')' * len(comp(miRNA4[7:])) + '.' * len(comp(miRNA4[:7]))
    

    structure_map += ('.' * len(Toehold) +
                     '(' * 9 + '.' * 3 + '(' * 6 + '.' * 15 + ')' * 6 + '.' * 3 + ')' * 9 + '.' * 9) #Riboswitch internal structure

    key_length = len(key_strand)

    domain_order = key_strand + domain_order  #Added key strand

    


    if len(domain_order) != len(structure_map):
        print('Error: domain order and structure map are not the same length')
        print(domain_order)
        print(structure_map)

    '''
    displaced_strand = (miRNA_1 + poly_a_domain + miRNA_2 + toehold).upper()
    print(structure_map)
    displaced_strand = (miRNA_1 + poly_a_domain + miRNA_2 + toehold).upper()
    

    
    if algorithm_selector == 'Road':
        print("Shoot")
        sequence = generate_road(
                structure=structure_map,
                sequence=domain_order,
                verbose=True,
                directory="../.temp/road"
            )
        print(sequence)
        print("Hit")

        test_seqs = f"{displaced_strand}&{sequence}"
        fc = RNA.fold_compound(test_seqs)
        structure, mfe = fc.mfe()
    
        print("Sequence :", test_seqs)
        print("Structure:", structure)
        print("MFE      :", mfe, "kcal/mol")
        return [sequence, structure_map, structure, mfe]
    '''
    if algorithm_selector == 'Hill':
        results = multi_start_hill_climb(domain_order, structure_map, get_pair_map(structure_map, domain_order), [i for i, x in enumerate(domain_order) if x == "N" or x == "K" or x == "k"])
        best_result = min(results, key=lambda x: x[3])
        print(structure_map)
        with open("../data/run15.txt", "w") as f:
            for res in results:
                 f.write(str(res))
                 f.write("\n")
                 f.write("Key strand: " +res[1][:key_length])
                 f.write("\n")
                 f.write("Main strand: " +res[1][key_length:])
                 f.write("\n")
                 f.write("\n")
        ''' Linnea 1
        for res in results:
            print(res)
            toehold_start_idx = len(miRNA1[7:] + comp(RiboSwitch[:15]) + comp(Toehold2) + RiboSwitch[7:15] + comp(miRNA1[7:]))
            print(toehold_start_idx)
            Toehold = res[1][toehold_start_idx:(toehold_start_idx + 10)] #Make it more robust
            key_strand = (comp(Toehold) + miRNA1[7:] + comp(RiboSwitch[7:15]) + miRNA2[7:])
            print(key_strand)
            test_seqs = f"{key_strand}&{res[1]}"
            fc = RNA.fold_compound(test_seqs)
            structure, mfe = fc.mfe()
            print(structure)
            print(mfe)
        '''
       
        

        return [best_result[1], structure_map, best_result[2], best_result[3]]


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
        print(current_line)
        origami_motif_build.append(current_line)
        current_line.clear()
        current_line = [pf.Dovetail(0, down_cross=False)]



    '''origami2 = pf.Origami([pf.Stem(sequence=sequence[:17]), pf.single_strand(sequence[17:24]),
    pf.Stem(sequence=sequence[24:38]), pf.single_strand(sequence[38:45]), pf.Stem(sequence=sequence[45:60]), pf.Loop(open_left=True,sequence=sequence[60:85])])
    '''
    print(origami)
    print(origami_motif_build)
    origami = origami_motif_build
    path = "../data/pdb/" + str(random.random())[2:]
    origami.save_3d_model(path, pdb=True)
    return jsonify({"path": path[2:]})

@app.route('/data/pdb/<path:filename>', methods=['GET'])
@cross_origin()
def serve_pdb(filename):
    PDB_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'data', 'pdb'))
    try:    
        return send_from_directory(PDB_DIR, filename, mimetype='chemical/x-pdb', as_attachment=True, download_name='structure.pdb')
    except FileNotFoundError:
        abort(404)


@app.route("/target", methods=['GET'])
def target_disease():
    query = request.args.get('query')
    df = pd.read_excel('../data/HMDD.xlsx')
    diease_assoc_rows = df.loc[df['disease'] == query]
    miRNAs = diease_assoc_rows['miRNA'].tolist()
    descriptions = diease_assoc_rows['description'].tolist()
    return jsonify({"results": miRNAs, "descriptions": descriptions})
