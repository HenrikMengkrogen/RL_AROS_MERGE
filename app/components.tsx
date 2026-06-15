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
  const idesignContext: IDesignContext = useContext(DesignContext);
  console.log(idesignContext.miRNA_list[0])
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
