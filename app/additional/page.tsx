"use client";
import {SpacerInputField, InitToeholdInputField, BDomainInputField, Toehold2InputField, RbsInputField} from "../components"

export default function Page() {
  return(<main className="flex min-h-screen flex-col items-center justify-between p-24"> 
  <h1>Additional parameters Page</h1>
    <SpacerInputField />
    <RbsInputField />
    <InitToeholdInputField />
    <BDomainInputField />
    <Toehold2InputField />
    </main>
  );
}