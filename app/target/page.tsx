"use client";
import { useState, useRef, useContext, useEffect } from "react";
import {MiRNATable} from '../components'
import Select from 'react-select';

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL

export default function Page() {
   interface Option {
  value: string;
  label: string;
}
let default_option: Option = {
    value: "",
    label: ""
}
  const [selectedTarget, setSelectedTarget] = useState("");
  const [miRNAResults, setmiRNAResults] = useState<string[]>([]);
  const [selectedOption, setSelectedOption] = useState<Option>(default_option);
  const [descriptions, setdescriptions] = useState<string[]>([]);

  useEffect(() => {
      const query = selectedTarget
        fetch(BACKEND_URL + `/target?query=${encodeURIComponent(query)}`)
          .then((res) => res.json())
          .then((data) => {
            setmiRNAResults(data.results ?? ["No matches found"]);
            setdescriptions(data.descriptions ?? ["No matches found"]);
            // idesignContext.setProtein(data.sequence); Set miRNAs
          })
          .catch(() => {setmiRNAResults(["Something went wrong"]); setdescriptions(["Something went wrong"]);});
    }, [selectedTarget]);

  function handleTargetChange(e: any) {
    e.preventDefault()
    setSelectedTarget(e.target.value)
  }
  function handleTargetChange2(e: any) {
    setSelectedOption(e)
    setSelectedTarget(selectedOption.value)
  }
  const options: Array<Option> = [
  { value: 'nonpapillary renal cell carcinoma', label: 'nonpapillary renal cell carcinoma' },
  { value: 'Myocardial Fibrosis', label: 'Myocardial Fibrosis' }
  ];
  return (
    <main className="flex min-h-screen flex-col items-center justify-around p-10">
        <h1>Target page</h1>
        <div className="w-80">
        <label>
        Pick a target:
        <Select<Option>
          value={selectedOption}
          onChange={(option) => handleTargetChange2(option)}
          options={options}
          isClearable
          isSearchable
          placeholder="Select a music genre..."
          aria-label="Music genre selector"
        />
        </label>
        </div>
        {(miRNAResults.length != 0) && <MiRNATable miRNAList={miRNAResults} description={descriptions}/>}

    </main>
  )}
