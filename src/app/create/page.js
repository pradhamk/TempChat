"use client";

import BackArrow from "@/components/BackArrow"
import { Input, Button } from "@nextui-org/react"
import { useState } from "react"
import { generateUsername } from "unique-username-generator"
import { TfiReload } from "react-icons/tfi";

export default function CreateChat() {
    const [randomName, setRandomName] = useState(generateUsername("", 5));
    const [username, setUsername] = useState(randomName);
    const [usernameError, setUsernameError] = useState(false);
    const [limit, setLimit] = useState(2);

    function submit() {
        console.log(username, limit)
    }

    return (
        <main>
            <BackArrow location={"/"} />
            <div className="h-[90vh] flex justify-center items-center flex-col">
                <h1 className="font-bold text-5xl mt-0">Create a Chat Room</h1>
                <Input  
                    isClearable
                    size="lg"
                    label="Username"
                    variant="faded"
                    description="If empty, random username will be assigned"
                    placeholder={randomName}
                    className="max-w-[40vw] mt-10"
                    isInvalid={usernameError}
                    onValueChange={(e) => {
                        if(e.length > 15) {
                            setUsernameError(true);
                            return
                        }
                        if(usernameError) {
                            setUsernameError(false);
                        }
                        if(e.length === 0) {
                            setUsername(randomName)
                        } else {
                            setUsername(e)
                        }
                    }}
                />
                <Input 
                    size="lg"
                    label="Chat Limit"
                    variant="faded"
                    description="How many users can join"
                    className="max-w-[40vw] mt-10"
                    placeholder="2"
                    type="number"
                    onValueChange={(e) => {
                        if(e.length === 0) {
                            setLimit(2)
                        } else {
                            setLimit(parseInt(e))
                        }
                    }}
                />
                <Button color="primary" className="mt-3 pr-10 pl-10 pt-6 pb-6 font-bold" onClick={submit}>Create</Button>
            </div>
        </main>
    )
}