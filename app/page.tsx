import Link from 'next/link';

export default function Home() {
   return (
   <main className="flex min-h-screen flex-col justify-between items-center justify-around p-10">
    <h1>Home Page</h1>
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
