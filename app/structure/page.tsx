"use client";
import { SequenceInput} from "../components"

export default function Structure() {
   return (
   <main className="flex min-h-screen flex-col justify-between items-center p-10">
    <h1 className="mb-4 text-2xl font-bold tracking-tight text-heading md:text-4xl lg:text-5xl">Structure Page</h1>
    <p>Here you can fully customise the sequence to be generated. You can start from one of our templates, or just work from an empty start. You can add new parts to the sequence by clicking the "+" buttons on the two ends of the sequence. 
 In each part, the input at the top corresponds to the name of the part, it can be anything, as it only helps make the design clearer.
 The one in the middle is the sequence constraint, it can be a concrete RNA sequence, an unconstrained part denoted by 'N'-s, or a "named region" that is an unconstrained region that can appear multiple times in the sequence, and the algorithm will generate identical sequences for each occurrence. It can be denoted by any letter except A, G, C, T, N, or letters used for different named regions in the same design.
 The input at the bottom is for defining the intended structure in dot-bracket notation. 
    </p>
    <SequenceInput />
    </main>
    )}