"use client";

import Image from 'next/image'
import React, { useState, useEffect, useCallback } from "react";
import {useInputmiRNA, MiRNAInputField} from "./components"



const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL


//function post_design_request(miRNA1) {

//}


export default function Home() {
    //input declarations
   const miRNAInput1 = useInputmiRNA();
   const miRNAInput2 = useInputmiRNA();
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
                    miRNA1:miRNAInput1.input_text,
                    miRNA2:miRNAInput2.input_text
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
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
        <MiRNAInputField {...miRNAInput1} />
        <hr />
        <MiRNAInputField {...miRNAInput2} />
    <button onClick={handleButtonSubmit} className="py-4 px-4 rounded-full transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">Construct the design!</button>
     <h1>Construct outputs</h1>
     <p>{response.sequence}</p>
     <p>{response.dotB}</p>
    </main>
  )
}
