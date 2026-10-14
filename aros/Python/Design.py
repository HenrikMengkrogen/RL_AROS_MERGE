from pylab import *
import numpy as np
import os


def main():

    TRIAL_MODE = True
    

    input_file = 'motifs.txt'
    path_to_file = '../misc/'
    
    
    
    
    

    

    if TRIAL_MODE != True:
        save_results(path_to_file, design(read_textfile(path_to_file+input_file)))
        print('PYTHON: TRIAL MODE INACTIVE -> OUTPUT IS GENERATED')
    else:
        print("PYTHON: TRIAL MODE ACTIVE -> NO OUTPUT GENERATED")


def comp(seq): # This is just generate the complementary domains -> since the script went trough a couple of iterations there are some redundancy in some of the functions, but I will clean that up later
    transcribe_dict = {
        'A': 'U',
        'U': 'A',
        'G': 'C',
        'C': 'G'
    }
    rna_seq = ''
    for n in seq:
        if n in transcribe_dict:
            rna_seq += transcribe_dict[n]
        else:
            rna_seq += n.swapcase()
    return rna_seq[::-1]



def design(domains): # fixed design
    miRNA_1 = domains[0] 
    miRNA_2 = domains[1]
    spacer_1 = 'N'*4
    spacer_2 = 'N'*3
    spacer_3 = 'N'*2
    rbs_site = 'AGGAGGACAGCUAUG'
    miRNA1_th = comp(miRNA_1[:1]) 
    miRNA2_th = comp(miRNA_2[:1]) 
    miRNA_1 = miRNA_1[2:] 
    miRNA_2 = miRNA_2[2:]
    
    
    stem = 'N'*8 
    poly_c_domain = 'C'*3
    poly_g_domain = 'G'*3
    clamp = 'N'*3

    
    
    
    
    poly_u_domain = 'U'*3
    
    
    
    
    stem_spacing = 10
    
    mrna = 'AAA'




    
        
    



    domain_order = (miRNA_1 + poly_c_domain + miRNA_2 + spacer_1 + miRNA2_th + comp(miRNA_2) + spacer_2 + miRNA1_th + comp(miRNA_1) + 'N'*stem_spacing + comp(miRNA_2) + poly_g_domain + comp(miRNA_1) + comp(clamp) + spacer_1 + stem + rbs_site + comp(stem) + 'AUG' + spacer_3 + clamp + miRNA_1 + poly_u_domain +   
            'N'*3 + comp(miRNA_1[:6]) + comp(clamp) + mrna

    ).upper()
    

    structure_map = (
        len(miRNA_1)*'(' + len(poly_c_domain)*'.' + len(miRNA_2)*'(' + len(spacer_1)*'.' + len(miRNA2_th)*'.' + len(miRNA_2)*')' + len(spacer_2)*'.' + len(miRNA1_th)*'.' + len(miRNA_1)*')' + stem_spacing*'.' + 
        len(comp(miRNA_2))*'.' + len(poly_g_domain)*'(' + len(miRNA_1)*'(' + len(clamp)*'(' + len(spacer_1)*'.' + len(stem)*'(' + len(rbs_site)*'.' + len(stem)*')' + '.'*3 + len(spacer_3)*'.' + len(clamp)*')' + len(miRNA_1)*')' + len(poly_u_domain)*')' +
        '.'*3 + len(miRNA_1[:6])*'.' + len(clamp)*'.' + len(mrna)*'.'
        ).upper()

    



    

    if len(domain_order) != len(structure_map):
        print('Error: domain order and structure map are not the same length')
    

    return np.array([domain_order, structure_map])


def read_textfile(path_to_file):
    
    mirna_1 = ""
    mirna_2 = ""

    with open(path_to_file, "r", encoding="utf-8") as file:
        for line in file:
            
            if not line.strip():
                continue
            
            
            key, value = line.split("=")
        
            
            key = key.strip()
            value = value.strip()
        
            
            if "miRNA_1" in key:
                mirna_1 = value
            elif "miRNA_2" in key:
                mirna_2 = value


    print("Sequence of miRNA 1:", mirna_1)
    print("Sequence of miRNA 2:", mirna_2)

    return np.array([mirna_1, mirna_2])


def save_results(path_to_file, Design): 
    Domain_order = Design[0]
    structure_map = Design[1]
    

    
    with open(os.path.join(path_to_file, 'input.txt'), 'w') as f:
        f.write(f'Sequence : {Domain_order}\n')
        f.write(f'Structure : {structure_map}')

    print(f'Saved to {path_to_file}/input.txt')




def temp_func():
    count_L = 0
    count_R = 0
    dot = 0

    struct = '(((((((((.(.......(((((((((((....)))))))))..))..........).)))))))))'
    for n in struct:
        if n == '(':
            count_R += 1
        if n == ')':
            count_L += 1
        else:
            dot += 1

    if count_L != count_R:
        print('STRUCTURE IS NOT BALANCED')

    else:
        print("STRUCTURE IS BALANCED")

    
    


if __name__ == '__main__':
    main()