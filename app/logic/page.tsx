export default function Page() {
  return (<main className="flex w-full min-h-screen flex-col items-center justify-around p-10">
    <h1>Logic Page</h1>
    <div className="flex flex-grow w-full flex-row justify-around p-10">
      <div className="flex flex-grow flex-col p-10 justify-around">
        <div className="bg-white rounded-xl bg-[#ccc]" draggable>
          <p className="text-center p-10">AND gate</p>
        </div>
        <div className="bg-white rounded-xl bg-[#ccc]" draggable>
          <p className="text-center p-10">OR gate</p>
        </div>
      </div>
      <div className="flex flex-grow w-3/4 p-10 bg-white">

      </div>
    </div>


    </main>);
}