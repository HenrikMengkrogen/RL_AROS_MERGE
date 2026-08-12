import React from 'react';
import { useState, useRef, useContext, useEffect } from "react";
import { IDesignContext, DesignContext } from './designContext';
import Image from 'next/image';
import img from '../public/logo.svg'

import { PluginContext } from 'molstar/lib/mol-plugin/context';
import { DefaultPluginSpec } from 'molstar/lib/mol-plugin/spec';

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
  const [protName, setProtName] = useState("");
  const debounceRef = useRef<number>();

   useEffect(() => {
    const query = protName
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
          idesignContext.setProtein(data.sequence);
        })
        .catch(() => { setSuggestions(["Something went wrong"]);});
    }, 350);
    return () => window.clearTimeout(debounceRef.current);
  }, [protName]);

  function handleChange(e: any) {
    setProtName(e.target.value)
    }

    function handleSuggestionClick(idx: number) {
      setProtName(suggestions[idx])
      setSuggestions([]);
    }
  return(
    <div className='flex flex-grow flex-col justify-around'>
    <div className='flex flex-col max-w-1/2 text-wrap'>
          <label> 
          Enter a protein name:
          </label>
          <input
          name='inp_sequence'
            type="text"
            value={protName ?? ""}
            onChange={handleChange}
          />
        
      <div className='flex flex-col divide-y-2 border-2'>
        {suggestions.map((suggestion, idx)=> <button className='bg-white p-2' key={idx} onClick={() => handleSuggestionClick(idx)}>{suggestion}</button>)}
      </div>
      </div>
      <div className='flex bg-white h-80 overflow-y-auto'>
        <p className='break-all'>{idesignContext.protein}</p>
      </div>
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
  console.log(response)
  console.log(response.response)
  console.log(response.response.sequence)
  return (
    <>
    <h2>Designed sequence: </h2>
    <div className='flex bg-white max-h-80 overflow-y-auto'>    
        <p className='break-all'>{response.response.sequence}</p>
    </div>
     <p>Original expected structure: {response.response.original_exp_structure}</p>
     <p>Structure with displacemenet strand: {response.response.disp_structure}</p>
     <p>MFE: {response.response.mfe}</p>
     </>
  );
}


export function Viewer3d({ url = 'https://files.rcsb.org/download/1CRN.cif' }) {
  const parentRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const pluginRef = useRef<PluginContext | null>(null);
  const [ready, setReady] = useState(false);

  // 1) One-time init. Empty deps → runs once for the component's life.
  useEffect(() => {
    const canvas = canvasRef.current;
    const parent = parentRef.current;
    if (!canvas || !parent) return;

    let disposed = false;
    const plugin = new PluginContext(DefaultPluginSpec());

    (async () => {
      await plugin.init();
      if (disposed) { plugin.dispose(); return; }

      const ok = await plugin.initViewerAsync(canvas, parent);
      if (disposed) { plugin.dispose(); return; }
      if (!ok) { console.error('Failed to init Mol* viewer'); return; }

      pluginRef.current = plugin;
      setReady(true);
    })();

    return () => {
      disposed = true;
      plugin.dispose();
      pluginRef.current = null;
    };
  }, []);

  // 2) Load structure whenever url changes (or once ready). No re-init.
  useEffect(() => {
    const plugin = pluginRef.current;
    if (!ready || !plugin) return;

    let cancelled = false;
    (async () => {
      await plugin.clear(); // remove previous structure before loading the new one
      if (cancelled) return;

      const data = await plugin.builders.data.download({ url }, { state: { isGhost: true } });
      if (cancelled) return;
      const format = url.endsWith('.cif') ? 'mmcif' : 'pdb';
      const trajectory = await plugin.builders.structure.parseTrajectory(data, format);
      if (cancelled) return;
      await plugin.builders.structure.hierarchy.applyPreset(trajectory, 'default');
    })();

    return () => { cancelled = true; };
  }, [url, ready]);

  return (
    <div
    ref={parentRef}
    style={{ position: 'relative', width: '100%', height: '600px', overflow: 'hidden' }}
    >
    <canvas ref={canvasRef} style={{ display: 'block' }} />
    </div>
  );
}