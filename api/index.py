from flask import Flask, jsonify
from flask import request
import RNA

app = Flask(__name__)


@app.route("/construct", methods=['GET', 'POST'])
def python_route():
    if request.method == 'GET':
        print("GET request received")
        return jsonify({"sequence": "AAAAAAA", 
                "dotB": "(..(..).)" })
    

    if request.method == 'POST':
        req_data = request.get_json()
        #TODO Validate incoming data
        [res_sequence, res_structure, disp_structure, mfe] = construct_design([req_data['miRNA1'], req_data['miRNA2'], req_data['spacer'], req_data['rbs_site'], req_data['init_toehold'], req_data['b_domain'], req_data['protein'], req_data['toehold2']])

        return jsonify({"sequence": res_sequence, 
                "original_exp_structure": res_structure,
                "disp_structure": disp_structure,
                "mfe": mfe})
    


from pyfurnace.generate.road import generate_road

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
            rna_seq += n.swapcase()
    return rna_seq[::-1]


def construct_design(domains):
    miRNA_1 = domains[0] 
    miRNA_2 = domains[1]
    miRNA1_th = comp(miRNA_1[:1]) 
    miRNA2_th = comp(miRNA_2[:1]) 
    spacer_1 = domains[2] 
    spacer_2 = spacer_1[:2] 
    spacer_3 = spacer_1[:1] 
    rbs_site = domains[3] 
    # miRNA_1 = miRNA_1[2:] TODO add custom slicing of the miRNAs
    # miRNA_2 = miRNA_2[2:]
    initial_toehold = domains[4] 
    B_domain = domains[5] # This domain together with the miRNA_1 invades into the 3rd loop and the initial toehold to open it up using LMSD, it is also used to control the translation of the mrna, it is also used to control the translation of the mrna, it is also used to control the translation of the mrna
    mrna = domains[6]
    Toehold_2 = domains[7]
    clamp_length = 3
    Comp_to_rbs = 'N' * ((len(rbs_site) - len(miRNA_1)) - clamp_length) + comp(miRNA_1) # Blocks RBS site, but also contains sequence for binding of miRNA_1, it is used to control the translation of the mrna
    poly_a_domain = 'A'*len(spacer_2)
    middle_stem = 'N'*(len(miRNA_1) + len(miRNA_2))
    clamp = rbs_site[-clamp_length:]
    mod_rbs_site = rbs_site[:len(rbs_site)-clamp_length]
    stem_spacing = 20
    
    domain_order = (initial_toehold + B_domain + miRNA_1 + spacer_2 + miRNA_2 + spacer_1 + comp(Toehold_2) + comp(miRNA_2) + comp(poly_a_domain) +
    comp(miRNA_1) + 'N'*stem_spacing + middle_stem + spacer_2 + miRNA1_th + comp(miRNA_1) + spacer_2 + miRNA2_th + comp(miRNA_2) + spacer_2 + middle_stem + 'N'*stem_spacing +
    comp(clamp) + Comp_to_rbs + comp(B_domain) + comp(initial_toehold) + spacer_1 + B_domain + mod_rbs_site + clamp + mrna).upper()

    structure_map = (
        len(initial_toehold)*'.' + len(B_domain)*'.' + len(miRNA_1)*'(' + len(spacer_2)*'.' + len(miRNA_2)*'(' + len(spacer_1)*'.' + len(Toehold_2)*'.' + len(comp(miRNA_2))*')' + 
    len(poly_a_domain)*'.' + len(miRNA_1)*')' + '.'*stem_spacing + '('*len(middle_stem) + '.'*len(spacer_2) + '.'*len(miRNA1_th) + '.'*len(miRNA_1) + '.'*len(spacer_2) + '.'*len(miRNA2_th) + 
    '.'*len(miRNA_2) + len(spacer_2)*'.' + ')'*len(middle_stem) + '.'*stem_spacing + '('*len(clamp) + '.'*len(Comp_to_rbs) + '(' *len(B_domain) + '.'*len(initial_toehold) + 
    '.'*len(spacer_1) + ')'*len(B_domain) + '.'*len(mod_rbs_site) + ')'*len(clamp) + '.'*len(mrna)
    )

    if len(domain_order) != len(structure_map):
        print('Error: domain order and structure map are not the same length')


    print(structure_map)
    displaced_strand = (miRNA_1 + poly_a_domain + miRNA_2 + Toehold_2).upper()
    

    

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
