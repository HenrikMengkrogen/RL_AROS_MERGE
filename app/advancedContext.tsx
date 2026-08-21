"use client";

import { useState, createContext, Dispatch, SetStateAction} from 'react';


export interface IAdvancedContext {
  sequence: string,
  structure: string,
  setSequence: Dispatch<SetStateAction<string>>
  setStructure: Dispatch<SetStateAction<string>>
}

const AdvancedContext = createContext<IAdvancedContext>({sequence:"", structure:"", setSequence: () => {}, setStructure: () => {}})

const AdvancedProvider = ({ children }: { children: React.ReactNode})  => {
  const [sequence, setSequence] = useState("");
  const [structure, setStructure] = useState("");
  const value: IAdvancedContext = 
  {sequence,
    structure,
    setSequence,
    setStructure
  }

  return (
    <AdvancedContext.Provider value={value}>
      {children}
    </AdvancedContext.Provider>
  )
}

export { AdvancedProvider, AdvancedContext }