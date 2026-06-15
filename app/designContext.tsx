"use client";

import { useState, createContext, Dispatch, SetStateAction} from 'react';


export interface IDesignContext {
  miRNA_list: Array<string>,
  protein: string,
  spacer: string,
  rbs_site: string,
  init_toehold: string,
  b_domain: string,
  toehold2: string,
  setMiRNA_list: Dispatch<SetStateAction<Array<string>>>,
  setProtein: Dispatch<SetStateAction<string>>,
  setSpacer: Dispatch<SetStateAction<string>>,
  setRbs_site: Dispatch<SetStateAction<string>>,
  setInit_toehold: Dispatch<SetStateAction<string>>,
  setB_domain: Dispatch<SetStateAction<string>>,
  setToehold2: Dispatch<SetStateAction<string>>
}

//Default context values
const baseDesignContext: IDesignContext = {
    miRNA_list: ["", ""],
    protein: "",
    spacer: "",
    rbs_site: "",
    init_toehold: "",
    b_domain: "",
    toehold2: "",
    setMiRNA_list: () => {},
    setProtein: () => {},
    setSpacer: () => {},
    setRbs_site: () => {},
    setInit_toehold: () => {},
    setB_domain: () => {},
    setToehold2: () => {}

}

const DesignContext = createContext<IDesignContext>(baseDesignContext);

const DesignProvider = ({ children }: { children: React.ReactNode})  => {
  const [_miRNA_list, _setMiRNA_list] = useState(["", ""]);
  const [_protein, _setProtein] = useState("");
  const [_spacer, _setSpacer] = useState("");
  const [_rbs_site, _setRbs_site] = useState("");
  const [_init_toehold, _setInit_toehold] = useState("");
  const [_b_domain, _setB_domain] = useState("");
  const [_toehold2, _setToehold2] = useState("");

  baseDesignContext.miRNA_list = _miRNA_list
  baseDesignContext.protein = _protein
  baseDesignContext.spacer = _spacer
  baseDesignContext.rbs_site = _rbs_site
  baseDesignContext.init_toehold = _init_toehold
  baseDesignContext.b_domain = _b_domain
  baseDesignContext.toehold2 = _toehold2

  baseDesignContext.setMiRNA_list = _setMiRNA_list
  baseDesignContext.setProtein = _setProtein
  baseDesignContext.setSpacer = _setSpacer
  baseDesignContext.setRbs_site = _setRbs_site
  baseDesignContext.setInit_toehold = _setInit_toehold
  baseDesignContext.setB_domain  = _setB_domain
  baseDesignContext.setToehold2 = _setToehold2

  return (
    <DesignContext.Provider value={baseDesignContext}>
      {children}
    </DesignContext.Provider>
  )
}

export { DesignContext, DesignProvider }



