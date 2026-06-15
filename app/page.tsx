"use client";

import Image from 'next/image'
import { Header } from './structure';
import React, { useState, useEffect, useCallback, useContext } from "react";
import {useInputmiRNA, MiRNAInputField} from "./components"
import { IDesignContext, DesignContext } from './designContext';


const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL


export default function Home() {
    //Read context
    const idesignContext: IDesignContext = useContext(DesignContext);
    //input declarations
   const miRNAInput1 = useInputmiRNA(0);
   const miRNAInput2 = useInputmiRNA(1);
   //output declarations
   const [submit, setSubmit] = useState(true)
   const [response, setdata] = useState({
        sequence: "",
        dotB: "",
    });
    


    const handleSubmit = useCallback(() => {
        fetch(BACKEND_URL + "/construct", {
                method: "POST",
                headers: { "Content-type": "application/json"},
                //Match inputs here
                body: JSON.stringify({
                    miRNA1:idesignContext.miRNA_list[0],
                    miRNA2:idesignContext.miRNA_list[1]
                })
            }).then((res) =>
                res.json().then((data) => {
                    // Setting a data from api
                    setdata({
                        sequence: data.sequence,
                        dotB: data.dotB,
                    });
                })
            );
    }, [submit])

  useEffect(handleSubmit, [handleSubmit]);


    function handleButtonSubmit(e: any) {
        setSubmit(!submit);
    }


  return (
    <div className='flex flex-row'>
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
        <MiRNAInputField {...miRNAInput1} />
        <hr />
        <MiRNAInputField {...miRNAInput2} />
    <button onClick={handleButtonSubmit} className="py-4 px-4 rounded-full transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">Construct the design!</button>
     <h1>Construct outputs</h1>
     <p>{response.sequence}</p>
     <p>{response.dotB}</p>
    </main>
    </div>
  )
}
