"use client";
import {SpacerInputField, InitToeholdInputField, BDomainInputField, Toehold2InputField, RbsInputField} from "../components"

export default function Page() {
  return(<div> <p>Additional parameters Page</p>
    <SpacerInputField />
    <RbsInputField />
    <InitToeholdInputField />
    <BDomainInputField />
    <Toehold2InputField />
    </div>
  );
}