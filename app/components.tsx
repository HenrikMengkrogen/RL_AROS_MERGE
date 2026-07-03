import React from 'react';
import { useContext, useEffect } from "react";
import { IDesignContext, DesignContext } from './designContext';
import Image from 'next/image';
import img from '../public/logo.svg'



export function MiRNAInputField({index} :{index : number}) {
  const idesignContext: IDesignContext = useContext(DesignContext);

  function handleChange(e: any) {
        //Get corresponding sequence
         const fetchData = async ()=>{
          console.log(1)
          let res = await fetch('https://www.rnacentral.org/api/v1/rna?format=json')
          console.log(2)
          if (res.ok) {
            let data = await res.json()
            console.log(data)
            console.log(3)
            const updated_miRNA_list = [...idesignContext.miRNA_list]
            updated_miRNA_list[index] = e.target.value
            console.log(updated_miRNA_list)
            idesignContext.setMiRNA_list(updated_miRNA_list);
          }
    }
      fetchData()
        //Update sequence to context
        const updated_miRNA_name_list = [...idesignContext.miRNA_name_list]
        updated_miRNA_name_list[index] = e.target.value
        console.log(updated_miRNA_name_list)
        idesignContext.setMiRNA_name_list(updated_miRNA_name_list);
    }

  return(
    <div>
    <form>
          <label> 
          Enter a miRNA sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.miRNA_name_list[index]}
            onChange={handleChange}
          />
        </label>
      </form>
      </div>
  );
}


export function ProteinInputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  function handleChange(e: any) {
    idesignContext.setProtein(e.target.value)
  }
  return(
    <div>
    <form>
          <label> 
          Enter a protein coding sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.protein}
            onChange={handleChange}
          />
        </label>
      </form>
      </div>
  );
}

export function SpacerInputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  function handleChange(e: any) {
    idesignContext.setSpacer(e.target.value)
  }
  return(
    <div>
    <form>
          <label> 
          Enter a spacer sequence or length:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.spacer}
            onChange={handleChange}
          />
        </label>
      </form>
      </div>
  );
}

export function RbsInputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  function handleChange(e: any) {
    idesignContext.setRbs_site(e.target.value)
  }
  return(
    <div>
    <form>
          <label> 
          Enter the ribosomal binding site sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.rbs_site}
            onChange={handleChange}
          />
        </label>
      </form>
      </div>
  );
}


export function ToeholdInputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  function handleChange(e: any) {
    idesignContext.setToehold(e.target.value)
  }
  return(
    <div>
    <form>
          <label> 
          Enter the toehold sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.toehold}
            onChange={handleChange}
          />
        </label>
      </form>
      </div>
  );
}


//Loading animation
export function LoadingIndicator() {

  return (
    <div>
    <p className='text-center'>Loading...</p>
    <Image
          className="flex animate-[spin_2000ms_linear_infinite]"
          src={img}
          alt="Team logo"
          width={150}
          height={150}
          priority
        />
    </div>
  );

}


export function DesignOutput(response : any) {
  return (
    <>
    <p>Designed sequence: {response.sequence}</p>
     <p>Original expected structure: {response.original_exp_structure}</p>
     <p>Structure with displacemenet strand: {response.disp_structure}</p>
     <p>MFE: {response.mfe}</p>
     </>
  );
}