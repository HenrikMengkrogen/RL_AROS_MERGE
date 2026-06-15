import React from 'react';
import Link from 'next/link';
import Image from 'next/image';
import img from '../public/logo_first.png'



function NavButton({ label }: { label: string }) {
  return (
      <Link href={(label == "Home") ? "/" : `/${label}`.split(" ")[0].toLowerCase()} className="flex w-full justify-center rounded-full bg-foreground py-8 mx-4 text-background transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]">

      <p>{label}</p>
      </Link>
  );
}

export function Header() {
  return (
    <div className="flex flex-col h-full items-center justify-center py-16 px-16 w-1/4">
      <Image
          className="flex py-8 px-8"
          src={img}
          alt="Team logo"
          width={150}
          height={150}
          priority
        />
      <NavButton label="Home" />
      <NavButton label="miRNA inputs" />
      <NavButton label="Logic gates" />
      <NavButton label="Protein output" />
      <NavButton label="Additional parameters" />
      <NavButton label="Design" />
    </div>
  );
}