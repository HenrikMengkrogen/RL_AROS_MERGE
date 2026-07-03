"use client";

import { useState, createContext, Dispatch, SetStateAction} from 'react';


export interface IDesignContext {
  miRNA_list: Array<string>,
  miRNA_name_list: Array<string>,
  protein: string,
  spacer: string,
  rbs_site: string,
  toehold: string,
  setMiRNA_list: Dispatch<SetStateAction<Array<string>>>,
  setMiRNA_name_list: Dispatch<SetStateAction<Array<string>>>,
  setProtein: Dispatch<SetStateAction<string>>,
  setSpacer: Dispatch<SetStateAction<string>>,
  setRbs_site: Dispatch<SetStateAction<string>>,
  setToehold: Dispatch<SetStateAction<string>>
}

//Default context values
const baseDesignContext: IDesignContext = {
    miRNA_list: ["", ""],
    miRNA_name_list: ["", ""],
    protein: "",
    spacer: "",
    rbs_site: "",
    toehold: "",
    setMiRNA_list: () => {},
    setMiRNA_name_list: () => {},
    setProtein: () => {},
    setSpacer: () => {},
    setRbs_site: () => {},
    setToehold: () => {}

}

const DesignContext = createContext<IDesignContext>(baseDesignContext);

const DesignProvider = ({ children }: { children: React.ReactNode})  => {
  const [_miRNA_list, _setMiRNA_list] = useState(["", ""]);
  const [_miRNA_name_list, _setMiRNA_name_list] = useState(["", ""]);
  const [_protein, _setProtein] = useState("");
  const [_spacer, _setSpacer] = useState("");
  const [_rbs_site, _setRbs_site] = useState("");
  const [_toehold, _setToehold] = useState("");

  baseDesignContext.miRNA_name_list = _miRNA_name_list
  baseDesignContext.miRNA_list = _miRNA_list
  baseDesignContext.protein = _protein
  baseDesignContext.spacer = _spacer
  baseDesignContext.rbs_site = _rbs_site
  baseDesignContext.toehold = _toehold

  baseDesignContext.setMiRNA_list = _setMiRNA_list
  baseDesignContext.setMiRNA_name_list = _setMiRNA_name_list
  baseDesignContext.setProtein = _setProtein
  baseDesignContext.setSpacer = _setSpacer
  baseDesignContext.setRbs_site = _setRbs_site
  baseDesignContext.setToehold = _setToehold

  return (
    <DesignContext.Provider value={baseDesignContext}>
      {children}
    </DesignContext.Provider>
  )
}

export { DesignContext, DesignProvider }



