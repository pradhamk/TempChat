"use client";

import { useSearchParams } from "next/navigation";
import { Divider, Button, Textarea } from "@nextui-org/react";
import { FaRegCopy } from "react-icons/fa";
import { IoMdSend } from "react-icons/io";
import { useState } from "react";
import { IoMdClose } from "react-icons/io";
import ChatBubble from "@/components/ChatBubble";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

export default function ChatRoom() {
    const search = useSearchParams()
    const room_url = search.get('roomURL')

    const [message, setMessage] = useState("")
    const [messages, setMessages] = useState([])

    useEffect(() => {
        if(!window) { return }
        const unlisten = listen('new-message', (e) => {
            const content = e.payload
            setMessages((prev) => [...prev, content])
        })

        return () => {
            unlisten.then(f => f())
        }
    }, [])

    return (
        <main className="w-full flex flex-col items-center">
            <div className="flex mt-8 mb-3 w-[80vw] items-center">
                <h1 className="text-xl font-bold text-center">Chat ID: 
                    <Button 
                        color="primary" 
                        variant="flat" 
                        className="ml-3 text-white" 
                        startContent={<FaRegCopy color="purple"/>}
                        onClick={() => { navigator.clipboard.writeText(room_url) }}
                    >
                        {room_url.split(".")[0].replace("https://", "")}
                    </Button>
                </h1>
                <Button color="danger" variant="ghost" className="ml-auto" size="sm">
                    <IoMdClose size={25}/>
                </Button>
            </div>
            
            <Divider className="w-[80vw]"/>
            <div className="w-[80vw] max-h-[80vh] pb-20 mt-3 overflow-scroll no-scrollbar">
                {
                    messages.map((val, i) => (
                        <ChatBubble time={new Date().toLocaleTimeString()} author={"user"} content={val} self={false} key={i}/>
                    ))
                }
            </div>
            <div className="flex items-center absolute bottom-0 pb-10 bg-background">
                <Textarea
                    placeholder="Send your message"
                    maxLength={1000}
                    maxRows={4}
                    minRows={1}
                    variant="faded"
                    value={message}
                    onValueChange={setMessage}
                    className="w-[80vw] mr-3"
                />
                <Button color="primary">
                    <IoMdSend className="size-5"/>
                </Button>
            </div>
        </main>
    )
}