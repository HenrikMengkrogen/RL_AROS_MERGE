import React from 'react';
import { useState} from "react";



export function useInputmiRNA() {
  const [input_text, setiText] = useState("");

  function handleChange(e: any) {
        setiText(e.target.value);
    }

  return {
        input_text,
        onChange: handleChange
    };
}

export function MiRNAInputField(inputProps: any) {
  return(
    <form>
          <label> 
          Enter a miRNA sequence:
          <input
          name='inp_sequence'
            type="text" 
            value={inputProps.input_text}
            onChange={inputProps.onChange}
          />
        </label>
      </form>
  );
}
