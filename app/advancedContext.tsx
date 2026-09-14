"use client";

import { useState, createContext, Dispatch, SetStateAction} from 'react';


export interface IAdvancedContext {
  name: Array<string>,
  sequence: Array<string>,
  structure: Array<string>,
  setSequence: Dispatch<SetStateAction<Array<string>>>
  setStructure: Dispatch<SetStateAction<Array<string>>>
  setName: Dispatch<SetStateAction<Array<string>>>
}

const AdvancedContext = createContext<IAdvancedContext>({name: [""], sequence:[""], structure:[""], setSequence: () => {},setStructure: () => {} , setName: () => {}})

const AdvancedProvider = ({ children }: { children: React.ReactNode})  => {
  const [name, setName] = useState([""]);
  const [sequence, setSequence] = useState([""]);
  const [structure, setStructure] = useState([""]);
  const value: IAdvancedContext = 
  { name,
    sequence,
    structure,
    setSequence,
    setStructure,
    setName
  }

  return (
    <AdvancedContext.Provider value={value}>
      {children}
    </AdvancedContext.Provider>
  )
}

export { AdvancedProvider, AdvancedContext }