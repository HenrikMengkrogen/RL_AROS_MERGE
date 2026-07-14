import React from 'react';
import { useState, useRef, useContext, useEffect } from "react";
import { IDesignContext, DesignContext } from './designContext';
import Image from 'next/image';
import img from '../public/logo.svg'

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL



export function MiRNAInputField({index} :{index : number}) {
  const idesignContext: IDesignContext = useContext(DesignContext);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [mirSeqFull, setMirSeqFull] = useState("");
  const debounceRef = useRef<number>();

   useEffect(() => {
    const query = idesignContext.miRNA_name_list[index]
    window.clearTimeout(debounceRef.current);
    
    if (query.length < 2) {
      setSuggestions([]);
      return;
    }

    debounceRef.current = window.setTimeout(() => {
      fetch(BACKEND_URL + `/suggest?query=${encodeURIComponent(query)}`)
        .then((res) => res.json())
        .then((data) => {
          setSuggestions(data.suggestions ?? ["No matches found"]);
          setMirSeqFull(data.sequence);
        })
        .catch(() => { setSuggestions(["Something went wrong"]);});
    }, 350);
    return () => window.clearTimeout(debounceRef.current);
  }, [idesignContext.miRNA_name_list[index]]);

  function handleChange(e: any) {

          const updated_miRNA_list = [...idesignContext.miRNA_list]
          updated_miRNA_list[index] = e.target.value
            //console.log(updated_miRNA_list)
            idesignContext.setMiRNA_list(updated_miRNA_list);
        //Update sequence to context
        const updated_miRNA_name_list = [...idesignContext.miRNA_name_list]
        updated_miRNA_name_list[index] = e.target.value
        console.log(updated_miRNA_name_list)
        idesignContext.setMiRNA_name_list(updated_miRNA_name_list);
    }

    function handleSuggestionClick(idx: number) {
      const updated_miRNA_name_list = [...idesignContext.miRNA_name_list]
      updated_miRNA_name_list[index] = suggestions[idx]
      idesignContext.setMiRNA_name_list(updated_miRNA_name_list)
      setSuggestions([]);
    }
  return(
    <div className='flex flex-col'>
          <label> 
          Enter a miRNA name:
          </label>
          <input
          name='inp_sequence'
            type="text"
            value={idesignContext.miRNA_name_list[index] ?? ""}
            onChange={handleChange}
          />
        
      <div className='flex flex-col divide-y-2 border-2'>
        {suggestions.map((suggestion, idx )=> <button className='bg-white p-2' key={idx} onClick={() => handleSuggestionClick(idx)}>{suggestion}</button>)}
      </div>
      <p>{mirSeqFull}</p>
      </div>
  );
}


export function ProteinInputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [mirSeqFull, setMirSeqFull] = useState("");
  const debounceRef = useRef<number>();

   useEffect(() => {
    const query = idesignContext.protein
    window.clearTimeout(debounceRef.current);
    
    if (query.length < 2) {
      setSuggestions([]);
      return;
    }

    debounceRef.current = window.setTimeout(() => {
      fetch(BACKEND_URL + `/mrna?query=${encodeURIComponent(query)}`)
        .then((res) => res.json())
        .then((data) => {
          setSuggestions(data.suggestions ?? ["No matches found"]);
          setMirSeqFull(data.sequence);
        })
        .catch(() => { setSuggestions(["Something went wrong"]);});
    }, 350);
    return () => window.clearTimeout(debounceRef.current);
  }, [idesignContext.protein]);

  function handleChange(e: any) {
    idesignContext.setProtein(e.target.value)
    }

    function handleSuggestionClick(idx: number) {
      setSuggestions([]);
    }
  return(
    <div className='flex flex-col'>
          <label> 
          Enter a mRNA name:
          </label>
          <input
          name='inp_sequence'
            type="text"
            value={idesignContext.protein ?? ""}
            onChange={handleChange}
          />
        
      <div className='flex flex-col divide-y-2 border-2'>
        {suggestions.map((suggestion, idx)=> <button className='bg-white p-2' key={idx} onClick={() => handleSuggestionClick(idx)}>{suggestion}</button>)}
      </div>
      <p>{mirSeqFull}</p>
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