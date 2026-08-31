"use client";

import { useState, createContext, Dispatch, SetStateAction} from 'react';


export interface IDesignContext {
  miRNA_list: Array<string>,
  miRNA_name_list: Array<string>,
  protein: string,
  setMiRNA_list: Dispatch<SetStateAction<Array<string>>>,
  setMiRNA_name_list: Dispatch<SetStateAction<Array<string>>>,
  setProtein: Dispatch<SetStateAction<string>>
}

//Default context values
const baseDesignContext: IDesignContext = {
    miRNA_list: ["", ""],
    miRNA_name_list: ["", ""],
    protein: "",
    setMiRNA_list: () => {},
    setMiRNA_name_list: () => {},
    setProtein: () => {}
}

const DesignContext = createContext<IDesignContext>(baseDesignContext);

const DesignProvider = ({ children }: { children: React.ReactNode})  => {
  const [miRNA_list, setMiRNA_list] = useState(["", ""]);
  const [miRNA_name_list, setMiRNA_name_list] = useState(["", ""]);
  const [protein, setProtein] = useState("");

  const value: IDesignContext = {
    miRNA_list,
    miRNA_name_list,
    protein,
    setMiRNA_list,
    setMiRNA_name_list,
    setProtein
  };

  return (
    <DesignContext.Provider value={value}>
      {children}
    </DesignContext.Provider>
  )
}

export { DesignContext, DesignProvider }



