"use client";
import { MiRNAInputField} from "../components"
import { IDesignContext, DesignContext } from '../designContext';
import { useState, useRef, useContext, useEffect } from "react";
import {MiRNATable} from '../components'
import AsyncSelect from 'react-select/async';
import disease_names from '../../data/HMDD_disease_names.json';


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
            })
            .catch(() => {setmiRNAResults(["Something went wrong"]); setdescriptions(["Something went wrong"]);});
      }, [selectedTarget]);
  
    function handleTargetChange(option: Option | null) {
      setSelectedOption(option ?? default_option)
      setSelectedTarget(option?.value ?? "")
    }
    const options: Array<Option> = disease_names.map(parse_option)
    function parse_option (name:string) : Option {
      return({value: name, label: name})
    }
    const loadOptions = (input: string) =>
    Promise.resolve(
      options
        .filter(o => o.label.toLowerCase().includes(input.toLowerCase()))
        .slice(0, 10)
    );

  return (
    <main className="flex min-h-screen flex-col justify-around p-10">
      <h1 className="mb-4 text-2xl font-bold tracking-tight text-heading md:text-4xl lg:text-5xl text-center">miRNA Page</h1>
      <p>Here you can look up disease-associated miRNAs from the HMDD database on the left and enter the chosen ones into the input fields on the right.
 The final construct will activate (expose the ribosome binding site) in the presence of both of these miRNAs. </p>
      <div className="flex  flex-row max-w-6xl justify-between gap-20">
        <div className="flex flex-grow flex-col min-w-[300px] w-3/4">
            <label>
            Pick a target:
            <AsyncSelect<Option>
              instanceId="target-select"
              value={selectedOption}
              onChange={(option) => handleTargetChange(option)}
              loadOptions={loadOptions}
              defaultOptions={options.slice(0, 10)}
              options={options}
              isClearable
              isSearchable
              placeholder="Select a disease..."
              aria-label="Target disease selector"
            />
            </label>
            {(miRNAResults.length != 0) && <MiRNATable miRNAList={miRNAResults} description={descriptions}/>}
        </div>
      <div className="min-w-[250px] w-1/4">
        <MiRNAInputField index={0} />
          <hr />
        <MiRNAInputField index={1} />
      </div>
      </div>
    </main>
  );
}