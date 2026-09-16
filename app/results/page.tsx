"use client";

import { useState, useEffect, useCallback, useContext } from "react";
import { IAdvancedContext, AdvancedContext } from '../advancedContext';
import { LoadingIndicator, DesignOutput, Viewer3d } from '../components'
import Link from 'next/link';
import Image from 'next/image';
import img from '../../public/file_download.svg'


const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL

export default function Page() {
   //Read context
   const iadvancedContext: IAdvancedContext = useContext(AdvancedContext);
   //output declarations
   const [submit, setSubmit] = useState(2)
   const [txt_link, setTxt_link] = useState("")
   const [pdb_link, setPdb_link] = useState("")
   const [response, setdata] = useState({
        sequence: "",
        generated_structure: "",
        mfe: "",
        filepath: ""
    });
    //Loading indicator
    const [isDataLoading, setIsDataLoading] = useState(false);
    const [receivedPDB, setReceivedPDB] = useState(false)
    
    const handleSubmit = useCallback(async () => {
        if (submit == 2) {
            return
        }
        else {
        setIsDataLoading(true)
        console.log(iadvancedContext.sequence)
        console.log(iadvancedContext.structure)
        await fetch(BACKEND_URL + "/advanced", {
                method: "POST",
                headers: { "Content-type": "application/json"},
                //Match inputs here
                body: JSON.stringify({
                    sequence: iadvancedContext.sequence.join(""),
                    structure: iadvancedContext.structure.join("")
                })
            }).then((res) =>
                res.json().then((data) => {
                    setdata({
                        sequence: data.sequence,
                        generated_structure: data.generated_structure,
                        mfe: data.mfe,
                        filepath: data.filepath
                    })
                    setTxt_link(response.filepath);
                    fetch(BACKEND_URL + "/pdb", {
                    method: "POST",
                    headers: { "Content-type": "application/json"},
                    //Match inputs here
                    body: JSON.stringify({
                    seq: data.sequence,
                    structure: data.generated_structure
                })
            }).then((res) => res.json().then((pdb_data) => setPdb_link(BACKEND_URL + pdb_data.path + ".pdb")))
                })
            );
            setIsDataLoading(false);
            
}} , [submit])

  useEffect(() => {handleSubmit()}, [handleSubmit]);


    function handleButtonSubmit(e: any) {
        setSubmit(submit == 2 ? 1 : -submit);
    }

  return (
    <main className="flex min-h-screen flex-col items-center justify-around p-10">
    <h1 className="mb-4 text-2xl font-bold tracking-tight text-heading md:text-4xl lg:text-5xl">Results page</h1>
    <button onClick={handleButtonSubmit} className="py-4 px-4 rounded-full transition-colors bg-white hover:bg-[#383838] dark:hover:bg-[#ccc]">Construct the design!</button>
     {isDataLoading && <LoadingIndicator />}
     {!isDataLoading && submit !=2 && 
     <DesignOutput sequence={response.sequence} gen_structure={response.generated_structure} original_structure={iadvancedContext.structure.join("")} mfe={response.mfe}/>}
    {/*<div className="h-50 max-h-50 p-10">
     {!isDataLoading && (pdb_link != "") && submit !=2 && (<div className="flex flex-col">
        <div className="flex bg-gray-400 rounded -mb-16 z-40 justify-end ml-auto">
     <Link href={pdb_link}>
     <Image
          src={img}
          alt="Team logo"
          width={50}
          height={50}
          priority />
     </Link>
    </div>
     <div className="z-0"><Viewer3d url={pdb_link} /></div>
     </div>)}
    </div>*/}
    </main>
  )
}