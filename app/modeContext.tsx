"use client";

import { useState, useEffect, createContext, Dispatch, SetStateAction} from 'react';


export interface IModeContext {
  isAdvancedMode: boolean,
  setisAdvancedMode: Dispatch<SetStateAction<boolean>>
}

let base_constant : IModeContext  = {
    isAdvancedMode: false,
    setisAdvancedMode: () => {}
}

const ModeContext = createContext<IModeContext>(base_constant);

const ModeProvider = ({ children, initialMode }: { children: React.ReactNode, initialMode:boolean})  => {
  const [isAdvancedMode, setisAdvancedMode] = useState(initialMode);

  const value: IModeContext = 
  {isAdvancedMode,
    setisAdvancedMode
  }
  useEffect(() => {
    cookieStore.set('mode', JSON.stringify(isAdvancedMode))
  }, [isAdvancedMode])
  
  return (
    <ModeContext.Provider value={value}>
      {children}
    </ModeContext.Provider>
  )
}

export { ModeContext, ModeProvider }