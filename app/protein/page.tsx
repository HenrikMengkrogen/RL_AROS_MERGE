"use client";
import {ProteinInputField} from "../components"

export default function Page() {
  return (
  <main className="flex min-h-screen flex-col items-center justify-around p-10">
    <h1 className="mb-4 text-2xl font-bold tracking-tight text-heading md:text-4xl lg:text-5xl">Protein Page</h1>
    <p>Here you can select the protein to be expressed when the construct gets activated. The search is based on UniProt.</p>
    <ProteinInputField />
  </main>


  );
}