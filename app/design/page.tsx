"use client";

import { useState, useEffect, useCallback, useContext } from "react";
import { IDesignContext, DesignContext } from '../designContext';
import { LoadingIndicator, DesignOutput, Viewer3d } from '../components'
import Link from 'next/link';
import Image from 'next/image';
import img from '../../public/file_download.svg'


const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL

export default function Page() {
   //Read context
   const idesignContext: IDesignContext = useContext(DesignContext);
   //output declarations
   const [submit, setSubmit] = useState(2)
   const [pdb_link, setPdb_link] = useState("")
   const [response, setdata] = useState({
        sequence: "",
        generated_structure: "",
        original_structure: "",
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

        await fetch(BACKEND_URL + "/construct", {
                method: "POST",
                headers: { "Content-type": "application/json"},
                //Match inputs here
                body: JSON.stringify({
                    miRNA1: idesignContext.miRNA_list[0],
                    miRNA2: idesignContext.miRNA_list[1],
                    protein: idesignContext.protein
                })
            }).then((res) =>
                res.json().then((data) => {
                    setdata({
                        sequence: data.sequence,
                        generated_structure: data.generated_structure,
                        original_structure: data.original_structure,
                        mfe: data.mfe,
                        filepath: data.filepath
                    });
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
    //Demo1 paramteres
    function handleDemo1() {
        idesignContext.setMiRNA_list(["UAGCUUAUCAGACUGAUGUUGACU", "UGGAGUGUGACAAUGGUGUUUG"])
        idesignContext.setProtein("")
    }

  return (
    <main className="flex min-h-screen flex-col items-center justify-around p-10">
    <button onClick={handleDemo1} className="py-4 px-4 rounded-full transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">Demo data 1</button>
    <button onClick={handleButtonSubmit} className="py-4 px-4 rounded-full transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">Construct the design!</button>
     <h1>Construct outputs</h1>
     {isDataLoading && <LoadingIndicator />}
     {!isDataLoading && submit !=2 && <DesignOutput sequence={response.sequence} gen_structure={response.generated_structure} original_structure={response.original_structure} mfe={response.mfe}/>}
    <div className="h-50 max-h-50 p-10">
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
    </div>

    </main>

  )
}