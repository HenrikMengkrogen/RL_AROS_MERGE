"use client";
import Link from 'next/link';
import { useState, useContext } from "react";
import { IModeContext, ModeContext } from './modeContext';

export default function Home() {
  const imodeContext: IModeContext = useContext(ModeContext);
   return (
   <main className="flex min-h-screen flex-col items-center justify-around p-10">
    <h1 className="mb-4 text-2xl font-bold tracking-tight text-heading md:text-4xl lg:text-5xl">Home Page</h1>
    <div className="flex gap-4">
      <span
        onClick={() => imodeContext.setisAdvancedMode(false)}
        className={`cursor-pointer px-2 py-1 rounded ${
          !imodeContext.isAdvancedMode
            ? "bg-blue-500 text-white"
            : "text-gray-500 hover:text-gray-700"
        }`}
      >
        Basic mode
      </span>
      <span
        onClick={() => imodeContext.setisAdvancedMode(true)}
        className={`cursor-pointer px-2 py-1 rounded ${
        imodeContext.isAdvancedMode
            ? "bg-blue-500 text-white"
            : "text-gray-500 hover:text-gray-700"
        }`}
      >
        Advanced mode
      </span>
    </div>
    <div className="flex flex-row">
        <Link href="https://2026.igem.wiki/aarhus-university/" className="flex w-full justify-center rounded-xl bg-foreground py-8 mx-4 text-background transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">
            <p className='p-5'>Tutorials</p>
        </Link>
        <Link href="https://2026.igem.wiki/aarhus-university/" className="flex w-full justify-center rounded-xl bg-foreground py-8 mx-4 text-background transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">
            <p className='p-5'>Documentation</p>
        </Link>
        <Link href="https://2026.igem.wiki/aarhus-university/" className="flex w-full justify-center rounded-xl bg-foreground py-8 mx-4 text-background transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">
            <p className='p-5'>iGEM Aarhus website</p>
        </Link>
    </div>
    </main>
    );
}
