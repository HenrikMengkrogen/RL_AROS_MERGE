import React from 'react';
import { useContext } from "react";
import { IDesignContext, DesignContext } from './designContext';



export function useInputmiRNA(index : number) {
  const idesignContext: IDesignContext = useContext(DesignContext);
  console.log(idesignContext.miRNA_list)

  function handleChange(e: any) {
        const updated_miRNA_list = [...idesignContext.miRNA_list]
        updated_miRNA_list[index] = e.target.value
        console.log(updated_miRNA_list)
        idesignContext.setMiRNA_list(updated_miRNA_list);
    }

  return {
        input_text: idesignContext.miRNA_list[index],
        onChange: handleChange
    };
}

export function MiRNAInputField(inputProps: any) {
  return(
    <div>
    <form>
          <label> 
          Enter a miRNA sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={inputProps.input_text}
            onChange={inputProps.onChange}
          />
        </label>
      </form>
      </div>
  );
}


export function ProteinInputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  function handleChange(e: any) {
    idesignContext.setProtein(e.target.value)
  }
  return(
    <div>
    <form>
          <label> 
          Enter a protein coding sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.protein}
            onChange={handleChange}
          />
        </label>
      </form>
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

export function InitToeholdInputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  function handleChange(e: any) {
    idesignContext.setInit_toehold(e.target.value)
  }
  return(
    <div>
    <form>
          <label> 
          Enter the initial toehold sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.init_toehold}
            onChange={handleChange}
          />
        </label>
      </form>
      </div>
  );
}

export function BDomainInputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  function handleChange(e: any) {
    idesignContext.setB_domain(e.target.value)
  }
  return(
    <div>
    <form>
          <label> 
          Enter the B Domain sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.b_domain}
            onChange={handleChange}
          />
        </label>
      </form>
      </div>
  );
}

export function Toehold2InputField() {
  const idesignContext: IDesignContext = useContext(DesignContext);
  function handleChange(e: any) {
    idesignContext.setToehold2(e.target.value)
  }
  return(
    <div>
    <form>
          <label> 
          Enter the second toehold sequence:
          <input
          name='inp_sequence'
            type="text"
            defaultValue={idesignContext.toehold2}
            onChange={handleChange}
          />
        </label>
      </form>
      </div>
  );
}
