"use client";
import {useContext } from "react";
import { MiRNAInputField} from "../components"
import { IDesignContext, DesignContext } from '../designContext';


export default function Page() {
  

  return (
    <main className="flex min-h-screen flex-col items-center justify-around p-10">
      <h1>miRNA Page</h1>
      <MiRNAInputField index={0} />
        <hr />
      <MiRNAInputField index={1} />
      </main>
  );
}