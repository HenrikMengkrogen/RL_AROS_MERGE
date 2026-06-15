"use client";
import {useContext } from "react";
import {useInputmiRNA, MiRNAInputField} from "../components"
import { IDesignContext, DesignContext } from '../designContext';


export default function Page() {
  //Init input varables
  const miRNAInput1 = useInputmiRNA(0);
  const miRNAInput2 = useInputmiRNA(1);
  

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <h1>miRNA Page</h1>
    <MiRNAInputField {...miRNAInput1} />
        <hr />
      <MiRNAInputField {...miRNAInput2} />
      </main>
  );
}