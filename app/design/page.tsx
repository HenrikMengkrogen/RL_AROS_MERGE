"use client";

import { useState, useEffect, useCallback, useContext } from "react";
import { IDesignContext, DesignContext } from '../designContext';
import { LoadingIndicator, DesignOutput } from '../components'

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL

export default function Page() {
   //Read context
    const idesignContext: IDesignContext = useContext(DesignContext);
   //output declarations
   const [submit, setSubmit] = useState(2)
   const [response, setdata] = useState({
        sequence: "",
        original_exp_structure: "",
        disp_structure: "",
        mfe: ""
    });
    //Loading indicator
    const [isDataLoading, setIsDataLoading] = useState(false);
    
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
                    protein: idesignContext.protein,
                    spacer: idesignContext.spacer,
                    rbs_site: idesignContext.rbs_site,
                    toehold: idesignContext.toehold
                })
            }).then((res) =>
                res.json().then((data) => {
                    // Setting a data from api
                    setdata({
                        sequence: data.sequence,
                        original_exp_structure: data.original_exp_structure,
                        disp_structure: data.disp_structure,
                        mfe: data.mfe
                    });
                })
            )

            setIsDataLoading(false);
}}, [submit])

  useEffect(() => {handleSubmit()}, [handleSubmit]);


    function handleButtonSubmit(e: any) {
        setSubmit(submit == 2 ? 1 : -submit);
    }
    //Demo1 paramteres
    function handleDemo1() {
        idesignContext.setMiRNA_list(["CUCUGAC", "CUCUUUC"])
        idesignContext.setMiRNA_name_list(["CUCUGAC", "CUCUUUC"])
        idesignContext.setSpacer("NNNN")
        idesignContext.setRbs_site("AGGAGGACAGCUAUG")
        idesignContext.setToehold("CA")
        idesignContext.setProtein("")
    }

  return (
    <div className='flex flex-row'>
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
    <button onClick={handleDemo1} className="py-4 px-4 rounded-full transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">Demo data 1</button>
    <button onClick={handleButtonSubmit} className="py-4 px-4 rounded-full transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">Construct the design!</button>
     <h1>Construct outputs</h1>
     {isDataLoading && <LoadingIndicator />}
     {!isDataLoading && submit !=2 && <DesignOutput response={response}/>}
    </main>
    </div>

  )
}