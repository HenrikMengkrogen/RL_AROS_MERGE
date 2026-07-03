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
        [res_sequence, res_structure, disp_structure, mfe] = construct_design([req_data['miRNA1'], req_data['miRNA2'], req_data['spacer'], req_data['rbs_site'], req_data['protein'], req_data['toehold']])

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
    stem = 'NNNNNN'
    mrna = domains[4]
    toehold = domains[5]
    poly_c_domain = 'C'*3
    loop = 'N'*6
    poly_g_domain = 'G'*3
    clamp = 'AGC'
    poly_a_domain = 'A'*len(spacer_2)
    poly_u_domain = 'U'*3
    middle_stem = 'N'*(len(miRNA_1) + len(miRNA_2))
    
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
        
    
    

    if len(domain_order) != len(structure_map):
        print('Error: domain order and structure map are not the same length')
    displaced_strand = (miRNA_1 + poly_a_domain + miRNA_2 + toehold).upper()
    print(structure_map)
    displaced_strand = (miRNA_1 + poly_a_domain + miRNA_2 + toehold).upper()
    

    

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
