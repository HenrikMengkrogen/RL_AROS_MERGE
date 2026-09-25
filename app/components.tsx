import React from 'react';
import { useState, useRef, useContext, useEffect } from "react";
import { IDesignContext, DesignContext } from './designContext';
import Image from 'next/image';
import img from '../public/logo.svg'
import { IAdvancedContext, AdvancedContext } from './advancedContext';
import { PluginContext } from 'molstar/lib/mol-plugin/context';
import { DefaultPluginSpec } from 'molstar/lib/mol-plugin/spec';
import { FaChevronDown, FaChevronUp } from "react-icons/fa";

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
          idesignContext.setMiRNA_list(prev => {          // write it here
            const next = [...prev];
            next[index] = data.sequence;
            return next;
          });
        })
        .catch(() => { setSuggestions(["Something went wrong"]);});
    }, 350);
    return () => window.clearTimeout(debounceRef.current);
  }, [idesignContext.miRNA_name_list[index]]);

  function handleChange(e: any) {
        //Update sequence to context
        const updated_miRNA_name_list = [...idesignContext.miRNA_name_list]
        updated_miRNA_name_list[index] = e.target.value
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
    <div className='flex flex-col justify-around'>
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
      {(idesignContext.protein != "") && <div className='flex bg-white h-80 overflow-y-auto'>
        <p className='break-all'>{idesignContext.protein}</p>
      </div>}
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

export function DesignOutput({sequence, gen_structure, original_structure, mfe} : {sequence: string, gen_structure: string, original_structure:string, mfe:string}) {
  return (
    <>
    <h2>Designed sequence: </h2>
    <div className='flex bg-white max-h-80 overflow-y-auto'>    
        <p className='font-mono break-all'>{sequence}</p>
    </div>
    <div className='flex flex-row '>
    <div className='flex flex-col'>
      <p>Generated structure:</p>
      <p>Original structure:</p>
    </div>
    <div className='min-w-0 flex-1 overflow-x-auto max-w-2xl bg-white'>
     <p className='font-mono text-sm whitespace-nowrap'>{gen_structure}</p>
     <p className='font-mono text-sm whitespace-nowrap'>{original_structure}</p>
    </div>
    </div>
     <p className='font-mono'>MFE: {mfe}</p>
     </>
  );
}


export function Viewer3d({ url = 'https://files.rcsb.org/download/1CRN.cif' }) {
  const parentRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const pluginRef = useRef<PluginContext | null>(null);
  const [ready, setReady] = useState(false);

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

  useEffect(() => {
    const plugin = pluginRef.current;
    if (!ready || !plugin) return;

    let cancelled = false;
    (async () => {
      await plugin.clear();
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
    style={{ position: 'relative', width: '100%', height: '400px', overflow: 'hidden' }}
    >
    <canvas ref={canvasRef} style={{ display: 'block' }} />
    </div>
  );
}

export function MiRNATable(response : {miRNAList : string[] , description : string[]}) {
  const miRNAList = response.miRNAList
  const description = response.description

  const rows = [];
  for (let i = 0; i < miRNAList.length; i++) {
    rows.push(
      <tr key={i}>
        <th scope='row' className='border border-gray-300'>
          {miRNAList[i]}
        </th>
        <td className='border border-gray-300'>{description[i]}</td>
      </tr>
    )
  }
 return(
  <div className='flex h-80 overflow-y-auto'>
  <table className=' border-separate border border-gray-500 '>
    <caption>
      Disease associated miRNAs returned from HMDD.
    </caption>
    <thead>
    <tr>
      <th scope="col" className='border border-gray-300 bg-[#ccc]'>miRNA name</th>
      <th scope="col" className='border border-gray-300 bg-[#ccc]'>Description</th>
    </tr>
  </thead>
  <tbody>
    {rows}
  </tbody>
  </table>
  </div>
 );
}
interface CardConent {
  seq_name: string,
  seq: string,
  structure: string
}

function SequenceComponents({items, setItems} : {items: Array<CardConent>, setItems: React.Dispatch<React.SetStateAction<CardConent[]>>}) {
  const itemCards = [];
  function handleNameChange(e : any, index : number) {
    let updated_items = [...items]
    updated_items[index].seq_name = e.target.value
    setItems(updated_items)
  }
  function handleSeqChange(e : any, index : number) {
    let updated_items = [...items]
    updated_items[index].seq = e.target.value
    setItems(updated_items)
  }
  function handleStructChange(e : any, index : number) {
    let updated_items = [...items]
    updated_items[index].structure = e.target.value
    setItems(updated_items)
  }
  function handleDelete(e : any, index : number) {
    let updated_items = [...items]
    updated_items.splice(index, 1)
    setItems(updated_items)
  }
  for (let i = 0; i < items.length; i++) {
    const width = Math.max(items[i].seq_name.length,items[i].seq.length, items[i].structure.length, 10);
    itemCards.push(
      <div key={i} className='flex flex-col items-stretch p-2 bg-[#ccc]'>
        <div className='flex flex-row font-mono'>
        <input size={width} className='text-center field-sizing-content'
          name='inp_sequence_name'
            type="text"
            value={items[i].seq_name}
            onChange={(e : any) =>handleNameChange(e, i)}
          />
          <button className='bg-white' onClick={(e : any) =>handleDelete(e, i)}>X</button>
          </div>
          <input size={width} className='flex text-center font-mono field-sizing-content'
          name='inp_sequence'
            type="text"
            value={items[i].seq}
            onChange={(e : any) =>handleSeqChange(e, i)}
          />
          <input size={width} className='flex text-center font-mono field-sizing-content'
          name='inp_structure'
            type="text"
            value={items[i].structure}
            onChange={(e : any) =>handleStructChange(e, i)}
          />
    </div>
   )
  }
  return(
  <div className='flex flex-row p-2 flex-wrap gap-y-8'>
    {itemCards}
  </div>);
}

export function SequenceInput() {
  const iadvancedContext: IAdvancedContext = useContext(AdvancedContext);
    let starter_items : Array<CardConent> = []
    for (let i = 0; i < iadvancedContext.name.length; i++) {
      starter_items.push({seq_name: iadvancedContext.name[i], seq: iadvancedContext.sequence[i], structure:iadvancedContext.structure[i]})
    }
    const [items, setItems] = useState(starter_items);
  useEffect(() => {
  iadvancedContext.setName(items.map(i => i.seq_name));
  iadvancedContext.setSequence(items.map(i => i.seq));
  iadvancedContext.setStructure(items.map(i => i.structure));
}, [items]);

    function handlePlusClickL(e : any) {
      const updated_items_list = [{seq_name: "", seq: "", structure:""}, ...items]
      setItems(updated_items_list)
    }
    function handlePlusClickR(e : any) {
      const updated_items_list = [ ...items, {seq_name: "", seq: "",structure:""}]
      setItems(updated_items_list)
    }
    function handleRiboTemplate() {
      setItems([{seq_name: "comp(miRNA1)", seq: "AGUCAACAUCAGUCUGAUAAGCUA", structure:"(((((((((((((((((......."}, 
        {seq_name: "comp(miRNA2)", seq: "CAAACACCAUUGUCACACUCCA", structure:"(((((((((((((((......."},
        {seq_name: "comp(RiboSwitchDisplace)", seq: "rrrrrrrrrrrrr", structure:"............."},
        {seq_name: "comp(Toehold)", seq: "kkkkkkkkkk", structure:".........."},
        {seq_name: "miRNA2[7:]", seq: "UGACAAUGGUGUUUG", structure:")))))))))))))))"},
        {seq_name: "loop7", seq: "NNNNNNN", structure:"......."},
        {seq_name: "miRNA1[7:]", seq: "UCAGACUGAUGUUGACU", structure:")))))))))))))))))"},
        {seq_name: "Toehold", seq: "KKKKKKKKKK", structure:".........."},
        {seq_name: "RiboSwitchDisplace", seq: "RRRRRRRRRRRRR", structure:"((((((((((((("},
        {seq_name: "Riboswitch", seq: "UUUUCCAUCAAGAACAGGCCACCAUGGAAAA", structure: "((((((((...............))))))))"},
        {seq_name: "comp(RiboSwitchDisplace)", seq: "rrrrrrrrrrrrr", structure:")))))))))))))"}
        ])
   }
   function handleEmptyTemplate() {
      setItems([{seq_name: "", seq: "", structure:""}
        ])
   }

    return(
    <>
    <TemplateDropdown buttonLabel="Templates"  items={[{title:"Empty template", action:handleEmptyTemplate}, {title:"RiboLogic template", action:handleRiboTemplate}]}/>
    <div className='flex flex-row items-center'>
      {(items.length > 0) && <button className='bg-[#ccc] p-2 w-12 h-12' onClick={handlePlusClickL}>+</button>}
      <SequenceComponents items={items} setItems={setItems}/>
      <button className='bg-[#ccc] p-2 w-12 h-12' onClick={handlePlusClickR}>+</button>
    </div>
    </>);
}

interface SingleLevelDropdownMenuProps {
  buttonLabel: string;
  setItems?: React.Dispatch<React.SetStateAction<CardConent[]>>;
  items: {
    title: string;
    icon?: JSX.Element;
    action?: () => void;
  }[];
}
export const TemplateDropdown = ({
  buttonLabel,
  items,
}: SingleLevelDropdownMenuProps) => {
  const [open, setOpen] = useState(false);
  const handleToggle = () => {
    setOpen((prev) => !prev);
  };
  return (
    <div className="relative">
      <button
        type="button"
        className="inline-flex items-center justify-center rounded-md text-sm border border-[#e4e4e7] h-10 px-4 py-2"
        onClick={handleToggle}
      >
        {buttonLabel}
        <span className="ml-2">
          {open ? <FaChevronUp /> : <FaChevronDown />}
        </span>
      </button>
      {open && (
        <div className="absolute left-1/2 -translate-x-1/2 top-12">
          <ul className="w-56 h-auto shadow-md rounded-md p-1 border bg-white">
            {items.map((item, index) => (
              <li
                key={index}
                className={`relative flex items-center gap-2 px-4 py-2 text-sm hover:bg-gray-100 rounded-md`}
              >
              <button className="w-full text-left"
                onClick={() => {
                  item.action?.();
                  setOpen(false);
                }}
              type="button"
              >
              {item.title}
              </button>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};
