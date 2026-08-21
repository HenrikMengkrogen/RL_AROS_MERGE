"use client";

import { useState, createContext, Dispatch, SetStateAction} from 'react';


export interface IModeContext {
  isAdvancedMode: boolean,
  setisAdvancedMode: Dispatch<SetStateAction<boolean>>
}

let base_constant : IModeContext  = {
    isAdvancedMode: false,
    setisAdvancedMode: () => {}
}

const ModeContext = createContext<IModeContext>(base_constant);

const ModeProvider = ({ children }: { children: React.ReactNode})  => {
  const [isAdvancedMode, setisAdvancedMode] = useState(false);

  const value: IModeContext = 
  {isAdvancedMode,
    setisAdvancedMode
  }

  return (
    <ModeContext.Provider value={value}>
      {children}
    </ModeContext.Provider>
  )
}

export { ModeContext, ModeProvider }