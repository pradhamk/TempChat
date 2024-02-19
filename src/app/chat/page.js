"use client";

import { useSearchParams } from "next/navigation";
import { Divider, Button, Textarea } from "@nextui-org/react";
import { FaRegCopy } from "react-icons/fa";
import { IoMdSend } from "react-icons/io";
import { useState } from "react";
import { IoMdClose } from "react-icons/io";
import ChatBubble from "@/components/ChatBubble";
import { listen, emit } from "@tauri-apps/api/event";
import { useEffect } from "react";
import Join from "@/components/Join";

export default function ChatRoom() {
    const search = useSearchParams()
    const room_url = search.get('roomURL')
    const username = search.get('username')

    const [message, setMessage] = useState("")
    const [messages, setMessages] = useState([])

    function sendMessage(e) {
        if(message.length === 0) {
            return
        }

        emit("host-message", { userMessage: { content: message } })
            .then(() => {
                console.log("hello")
                setMessage("")
            }).catch((e) => {
                console.log("Couldn't send message", e)
            })  
    }

    useEffect(() => {
        if(!window) { return }
        const message_unlisten = listen('new-message', (e) => {
            const content = JSON.parse(e.payload)
            setMessages((prev) => [...prev, content])
        })

        const join_unlisten = listen('join', (e) => {
            const content = JSON.parse(e.payload)
            console.log(content)
            setMessages((prev) => [...prev, content])
        })

        return () => {
            message_unlisten.then(f => f())
            join_unlisten.then(f => f())
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
                <Button 
                    color="danger" 
                    variant="ghost" 
                    className="ml-auto" 
                    size="sm"
                    onClick={() => {
                        emit("shutdown").then(() => {
                            //Shutdown modal closes
                            window.location.href = "/"
                        })
                    }}
                >
                    <IoMdClose size={25}/>
                </Button>
            </div>
            
            <Divider className="w-[80vw]"/>
            <div className="w-[80vw] max-h-[80vh] pb-20 mt-3 overflow-scroll no-scrollbar">
                {
                    messages.map((val, i) => {
                        if(val.joinMessage) {
                            return (<Join username={val.joinMessage.joined} key={i}/>)                            
                        } else {
                            val = val.broadcastMessage
                            return (<ChatBubble time={val.created} author={val.sender} content={val.content} self={val.sender === username ? true : false} key={i}/>)
                        }
                    })
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
                    onKeyDown={(e) => {
                        if(e.key === "Enter" && !e.shiftKey) {
                            e.preventDefault();
                            sendMessage()
                        }
                    }}
                />
                <Button color="primary" onClick={sendMessage} onPress={null}>
                    <IoMdSend className="size-5"/>
                </Button>
            </div>
        </main>
    )
}