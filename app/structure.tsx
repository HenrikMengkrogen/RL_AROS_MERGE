import React from 'react';
import Link from 'next/link';
import Image from 'next/image';
import img from '../public/logo.svg'



function NavButton({ label }: { label: string }) {
  return (
      <Link href={(label == "Home") ? "/" : `/${label}`.split(" ")[0].toLowerCase()} className="flex w-full justify-center rounded-full bg-foreground py-5 mx-4 text-background transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">

      <p>{label}</p>
      </Link>
  );
}

export function Header() {
  return (
    <div className="flex flex-col h-screen items-center justify-center gap-y-8 py-16 px-16  w-1/4 border-4 border-double">
      <div className=' bg-white rounded-full border-4 border-double'>
      <Image
          className="flex"
          src={img}
          alt="Team logo"
          width={150}
          height={150}
          priority
        />
      </div>
      <NavButton label="Home" />
      <NavButton label="Target" />
      <NavButton label="miRNA inputs" />
      <NavButton label="Logic gates" />
      <NavButton label="Protein output" />
      <NavButton label="Additional parameters" />
      <NavButton label="Design" />
    </div>
  );
}